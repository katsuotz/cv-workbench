use std::sync::Arc;

use argon2::{
    Argon2, Params, Version,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::RngCore;
use sha2::{Digest, Sha256};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use super::{
    model::{
        AccountResponse, AnonymousSessionResponse, AuthSessionResponse, Principal, UserRecord,
    },
    repository::SessionRepository,
};
use crate::error::AppError;

pub const AUTH_COOKIE: &str = "lr_session";
pub const ANONYMOUS_COOKIE: &str = AUTH_COOKIE;

pub struct SessionService {
    repository: Arc<dyn SessionRepository>,
    ttl: Duration,
}

impl SessionService {
    pub fn new(repository: Arc<dyn SessionRepository>, ttl: Duration) -> Self {
        Self { repository, ttl }
    }

    pub async fn create_anonymous(&self) -> Result<AnonymousSessionResponse, AppError> {
        let token = random_token();
        let hash = token_hash(&token);
        let expires_at = OffsetDateTime::now_utc() + self.ttl;
        let session_id = self.repository.create_anonymous(&hash, expires_at).await?;

        Ok(AnonymousSessionResponse {
            session_id,
            token,
            expires_at,
        })
    }

    pub async fn register(
        &self,
        email: &str,
        password: &str,
        display_name: Option<&str>,
        transfer_session_id: Option<Uuid>,
    ) -> Result<AuthSessionResponse, AppError> {
        let email = normalize_email(email)?;
        validate_password(password)?;
        let display_name = normalize_display_name(display_name)?;
        let password_hash = hash_password(password).await?;
        let token = random_token();
        let expires_at = OffsetDateTime::now_utc() + self.ttl;
        let user = self
            .repository
            .create_account(
                &email,
                &password_hash,
                display_name.as_deref(),
                &token_hash(&token),
                expires_at,
                transfer_session_id,
            )
            .await?;
        Ok(AuthSessionResponse {
            account: account_response(&user),
            token,
            expires_at,
        })
    }

    pub async fn login(
        &self,
        email: &str,
        password: &str,
    ) -> Result<AuthSessionResponse, AppError> {
        let email = normalize_email(email)?;
        validate_password(password)?;
        let user = self
            .repository
            .find_user_by_email(&email)
            .await?
            .ok_or(AppError::Unauthorized)?;
        let Some(password_hash) = user.password_hash.as_deref() else {
            return Err(AppError::Unauthorized);
        };
        if !verify_password(password, password_hash).await? {
            return Err(AppError::Unauthorized);
        }
        let token = random_token();
        let expires_at = OffsetDateTime::now_utc() + self.ttl;
        self.repository
            .create_auth_session(user.id, &token_hash(&token), expires_at)
            .await?;
        Ok(AuthSessionResponse {
            account: account_response(&user),
            token,
            expires_at,
        })
    }

    pub async fn authenticate(
        &self,
        headers: &axum::http::HeaderMap,
    ) -> Result<Principal, AppError> {
        if let Some(token) = cookie_value(headers, AUTH_COOKIE) {
            if let Some((auth_session_id, user_id)) = self
                .repository
                .find_active_auth_session(&token_hash(token))
                .await?
            {
                return Ok(Principal::User {
                    user_id,
                    auth_session_id,
                });
            }
        }

        if let Some(token) =
            bearer_token(headers).or_else(|| cookie_value(headers, ANONYMOUS_COOKIE))
        {
            return Ok(Principal::Anonymous {
                session_id: self
                    .repository
                    .find_active_anonymous(&token_hash(token))
                    .await?,
            });
        }

        Err(AppError::Unauthorized)
    }

    pub async fn logout(&self, headers: &axum::http::HeaderMap) -> Result<(), AppError> {
        if let Some(token) = cookie_value(headers, AUTH_COOKIE) {
            self.repository
                .revoke_auth_session(&token_hash(token))
                .await?;
        }
        if let Some(token) =
            bearer_token(headers).or_else(|| cookie_value(headers, ANONYMOUS_COOKIE))
        {
            self.repository
                .revoke_anonymous_session(&token_hash(token))
                .await?;
        }
        Ok(())
    }

    pub async fn account(&self, principal: &Principal) -> Result<AccountResponse, AppError> {
        let user_id = principal.user_id().ok_or(AppError::Unauthorized)?;
        Ok(account_response(
            &self.repository.find_user_by_id(user_id).await?,
        ))
    }

    pub async fn seed_user(
        &self,
        email: &str,
        password: &str,
    ) -> Result<AccountResponse, AppError> {
        let email = normalize_email(email)?;
        validate_password(password)?;
        let password_hash = hash_password(password).await?;
        Ok(account_response(
            &self.repository.seed_user(&email, &password_hash).await?,
        ))
    }
}

pub fn account_response(user: &UserRecord) -> AccountResponse {
    AccountResponse {
        id: user.id,
        email: user.email.clone(),
        display_name: user.display_name.clone(),
        created_at: user.created_at,
    }
}

pub fn normalize_display_name(name: Option<&str>) -> Result<Option<String>, AppError> {
    let Some(name) = name else {
        return Ok(None);
    };
    let name = name.trim();
    if name.is_empty() {
        return Ok(None);
    }
    if name.chars().count() > 256 {
        return Err(AppError::BadRequest(
            "name must be 256 characters or fewer".into(),
        ));
    }
    Ok(Some(name.to_owned()))
}

pub fn cookie_value<'a>(headers: &'a axum::http::HeaderMap, name: &str) -> Option<&'a str> {
    headers
        .get(axum::http::header::COOKIE)
        .and_then(|value| value.to_str().ok())
        .and_then(|cookies| {
            cookies.split(';').find_map(|pair| {
                let (key, value) = pair.trim().split_once('=')?;
                (key == name && !value.is_empty()).then_some(value)
            })
        })
}

pub fn bearer_token(headers: &axum::http::HeaderMap) -> Option<&str> {
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .filter(|value| !value.is_empty())
}

pub fn normalize_email(email: &str) -> Result<String, AppError> {
    let email = email.trim().to_ascii_lowercase();
    if email.len() < 3
        || email.len() > 320
        || email.bytes().filter(|byte| *byte == b'@').count() != 1
        || email.starts_with('@')
        || email.ends_with('@')
    {
        return Err(AppError::BadRequest("email is invalid".into()));
    }
    Ok(email)
}

fn validate_password(password: &str) -> Result<(), AppError> {
    if password.chars().count() < 12 || password.len() > 1024 {
        return Err(AppError::BadRequest(
            "password must be between 12 and 1024 characters".into(),
        ));
    }
    Ok(())
}

async fn hash_password(password: &str) -> Result<String, AppError> {
    let password = password.to_owned();
    tokio::task::spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        argon2()
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|error| AppError::Internal(format!("password hashing failed: {error}")))
    })
    .await
    .map_err(|error| AppError::Internal(format!("password hashing task failed: {error}")))?
}

async fn verify_password(password: &str, encoded_hash: &str) -> Result<bool, AppError> {
    let password = password.to_owned();
    let encoded_hash = encoded_hash.to_owned();
    tokio::task::spawn_blocking(move || {
        let parsed = PasswordHash::new(&encoded_hash).map_err(|error| {
            AppError::Internal(format!("stored password hash is invalid: {error}"))
        })?;
        Ok(argon2()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok())
    })
    .await
    .map_err(|error| AppError::Internal(format!("password verification task failed: {error}")))?
}

fn argon2() -> Argon2<'static> {
    let params = Params::new(19 * 1024, 2, 1, Some(32)).expect("valid Argon2id parameters");
    Argon2::new(argon2::Algorithm::Argon2id, Version::V0x13, params)
}

fn random_token() -> String {
    let mut bytes = [0_u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

pub(crate) fn token_hash(token: &str) -> [u8; 32] {
    Sha256::digest(token.as_bytes()).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_and_validates_email() {
        assert_eq!(
            normalize_email("  USER@Example.COM ").unwrap(),
            "user@example.com"
        );
        assert!(normalize_email("not-an-email").is_err());
    }

    #[test]
    fn parses_cookie_values_without_matching_prefixes() {
        let mut headers = axum::http::HeaderMap::new();
        headers.insert(
            axum::http::header::COOKIE,
            "other=x; lr_session=abc".parse().unwrap(),
        );
        assert_eq!(cookie_value(&headers, AUTH_COOKIE), Some("abc"));
        assert_eq!(cookie_value(&headers, "late"), None);
    }

    #[tokio::test]
    async fn hashes_passwords_with_argon2id_and_verifies_them() {
        let encoded = hash_password("correct horse battery staple").await.unwrap();
        assert!(encoded.starts_with("$argon2id$v=19$m=19456,t=2,p=1$"));
        assert!(
            verify_password("correct horse battery staple", &encoded)
                .await
                .unwrap()
        );
        assert!(!verify_password("wrong password", &encoded).await.unwrap());
    }
}
