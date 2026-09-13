use std::sync::Arc;

use async_trait::async_trait;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use rand::RngCore;
use reqwest::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use time::{Duration, OffsetDateTime};
use url::Url;
use uuid::Uuid;

use super::{
    model::{AuthSessionResponse, GoogleIdentity},
    repository::SessionRepository,
    service::{account_response, token_hash},
};
use crate::{config::GoogleConfig, error::AppError};

const GOOGLE_AUTHORIZATION_ENDPOINT: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_ENDPOINT: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_JWKS_ENDPOINT: &str = "https://www.googleapis.com/oauth2/v3/certs";
const GOOGLE_ISSUER: &str = "https://accounts.google.com";
pub const GOOGLE_STATE_COOKIE: &str = "lr_google_state";
const GOOGLE_STATE_TTL: Duration = Duration::minutes(10);

#[derive(Clone, Debug)]
pub struct GoogleAuthorization {
    pub url: String,
    pub state: String,
}

#[async_trait]
pub trait GoogleOidcProvider: Send + Sync {
    async fn exchange_code(
        &self,
        code: &str,
        code_verifier: &str,
        expected_nonce: &str,
    ) -> Result<GoogleIdentity, AppError>;
}

pub struct GoogleAuthService {
    repository: Arc<dyn SessionRepository>,
    provider: Option<Arc<dyn GoogleOidcProvider>>,
    config: Option<GoogleConfig>,
    ttl: Duration,
}

impl GoogleAuthService {
    pub fn new(
        repository: Arc<dyn SessionRepository>,
        ttl: Duration,
        config: Option<GoogleConfig>,
    ) -> Self {
        let provider = config.as_ref().map(|config| {
            Arc::new(GoogleProvider::new(config.clone())) as Arc<dyn GoogleOidcProvider>
        });
        Self {
            repository,
            provider,
            config,
            ttl,
        }
    }

    pub fn new_with_provider(
        repository: Arc<dyn SessionRepository>,
        ttl: Duration,
        config: GoogleConfig,
        provider: Arc<dyn GoogleOidcProvider>,
    ) -> Self {
        Self {
            repository,
            provider: Some(provider),
            config: Some(config),
            ttl,
        }
    }

    pub async fn start(
        &self,
        anonymous_session_id: Option<Uuid>,
    ) -> Result<GoogleAuthorization, AppError> {
        let config = self.config.as_ref().ok_or_else(|| {
            AppError::BadRequest("Google login is not configured on this server".into())
        })?;
        let state = random_token();
        let nonce = random_token();
        let pkce_verifier = random_token();
        self.repository
            .create_google_login_state(
                &token_hash(&state),
                &nonce,
                &pkce_verifier,
                OffsetDateTime::now_utc() + GOOGLE_STATE_TTL,
                anonymous_session_id,
            )
            .await?;

        let mut url = Url::parse(GOOGLE_AUTHORIZATION_ENDPOINT)
            .map_err(|error| AppError::Internal(error.to_string()))?;
        url.query_pairs_mut()
            .append_pair("client_id", &config.client_id)
            .append_pair("redirect_uri", &config.redirect_uri)
            .append_pair("response_type", "code")
            .append_pair("scope", "openid email profile")
            .append_pair("state", &state)
            .append_pair("nonce", &nonce)
            .append_pair("code_challenge", &pkce_challenge(&pkce_verifier))
            .append_pair("code_challenge_method", "S256");
        Ok(GoogleAuthorization {
            url: url.into(),
            state,
        })
    }

    pub async fn callback(
        &self,
        state: &str,
        state_cookie: Option<&str>,
        code: &str,
    ) -> Result<AuthSessionResponse, AppError> {
        validate_state_cookie(state, state_cookie)?;
        let login_state = self
            .repository
            .consume_google_login_state(&token_hash(state))
            .await?
            .ok_or(AppError::Unauthorized)?;
        let provider = self.provider.as_ref().ok_or_else(|| {
            AppError::BadRequest("Google login is not configured on this server".into())
        })?;
        let identity = provider
            .exchange_code(code, &login_state.pkce_verifier, &login_state.nonce)
            .await?;
        if !identity.email_verified {
            return Err(AppError::Unauthorized);
        }
        let email = super::service::normalize_email(&identity.email)?;
        let identity = GoogleIdentity { email, ..identity };
        let token = random_token();
        let expires_at = OffsetDateTime::now_utc() + self.ttl;
        let user = self
            .repository
            .complete_google_login(
                &identity,
                &token_hash(&token),
                expires_at,
                login_state.anonymous_session_id,
            )
            .await?;
        Ok(AuthSessionResponse {
            account: account_response(&user),
            token,
            expires_at,
        })
    }

    pub async fn cancel(&self, state: &str, state_cookie: Option<&str>) -> Result<(), AppError> {
        validate_state_cookie(state, state_cookie)?;
        self.repository
            .consume_google_login_state(&token_hash(state))
            .await?
            .ok_or(AppError::Unauthorized)?;
        Ok(())
    }
}

fn validate_state_cookie(state: &str, state_cookie: Option<&str>) -> Result<(), AppError> {
    if state.is_empty()
        || state_cookie
            .is_none_or(|cookie| !constant_time_equal(cookie.as_bytes(), state.as_bytes()))
    {
        return Err(AppError::Unauthorized);
    }
    Ok(())
}

pub fn state_cookie_header(state: &str, secure: bool) -> Result<axum::http::HeaderValue, AppError> {
    cookie_header(state, 600, secure)
}

pub fn clear_state_cookie_header(secure: bool) -> Result<axum::http::HeaderValue, AppError> {
    cookie_header("", 0, secure)
}

fn cookie_header(
    value: &str,
    max_age: u64,
    secure: bool,
) -> Result<axum::http::HeaderValue, AppError> {
    let secure_suffix = if secure { "; Secure" } else { "" };
    axum::http::HeaderValue::try_from(format!(
        "{GOOGLE_STATE_COOKIE}={value}; Path=/api/v1/auth/google; HttpOnly; SameSite=Lax; Max-Age={max_age}{secure_suffix}"
    ))
    .map_err(|error| AppError::Internal(error.to_string()))
}

fn random_token() -> String {
    let mut bytes = [0_u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

pub fn pkce_challenge(verifier: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}

fn constant_time_equal(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.iter()
        .zip(right)
        .fold(0_u8, |difference, (left, right)| {
            difference | (left ^ right)
        })
        == 0
}

struct GoogleProvider {
    client: Client,
    config: GoogleConfig,
}

impl GoogleProvider {
    fn new(config: GoogleConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    async fn verify_id_token(
        &self,
        id_token: &str,
        expected_nonce: &str,
    ) -> Result<GoogleIdentity, AppError> {
        let header = decode_header(id_token).map_err(|_| AppError::Unauthorized)?;
        if header.alg != Algorithm::RS256 {
            return Err(AppError::Unauthorized);
        }
        let kid = header.kid.ok_or(AppError::Unauthorized)?;
        let jwks = self
            .client
            .get(GOOGLE_JWKS_ENDPOINT)
            .send()
            .await
            .map_err(|_| AppError::Internal("Google identity verification failed".into()))?
            .error_for_status()
            .map_err(|_| AppError::Internal("Google identity verification failed".into()))?
            .json::<GoogleJwks>()
            .await
            .map_err(|_| AppError::Internal("Google identity verification failed".into()))?;
        let key = jwks
            .keys
            .into_iter()
            .find(|key| key.kid.as_deref() == Some(kid.as_str()))
            .ok_or(AppError::Unauthorized)?;
        let decoding_key =
            DecodingKey::from_rsa_components(&key.n, &key.e).map_err(|_| AppError::Unauthorized)?;
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&[GOOGLE_ISSUER, "accounts.google.com"]);
        validation.set_audience(std::slice::from_ref(&self.config.client_id));
        let claims = decode::<GoogleClaims>(id_token, &decoding_key, &validation)
            .map_err(|_| AppError::Unauthorized)?
            .claims;
        let issuer_is_valid = claims.iss == GOOGLE_ISSUER || claims.iss == "accounts.google.com";
        if !issuer_is_valid
            || claims.aud != self.config.client_id
            || claims.exp <= OffsetDateTime::now_utc().unix_timestamp() as usize
            || claims.nonce != expected_nonce
            || !claims.email_verified
            || claims.sub.is_empty()
        {
            return Err(AppError::Unauthorized);
        }
        Ok(GoogleIdentity {
            subject: claims.sub,
            email: claims.email,
            email_verified: claims.email_verified,
            display_name: claims.name.and_then(normalize_display_name),
        })
    }
}

#[async_trait]
impl GoogleOidcProvider for GoogleProvider {
    async fn exchange_code(
        &self,
        code: &str,
        code_verifier: &str,
        expected_nonce: &str,
    ) -> Result<GoogleIdentity, AppError> {
        let response = self
            .client
            .post(GOOGLE_TOKEN_ENDPOINT)
            .form(&[
                ("code", code),
                ("client_id", self.config.client_id.as_str()),
                ("client_secret", self.config.client_secret.as_str()),
                ("redirect_uri", self.config.redirect_uri.as_str()),
                ("grant_type", "authorization_code"),
                ("code_verifier", code_verifier),
            ])
            .send()
            .await
            .map_err(|_| AppError::Internal("Google authorization failed".into()))?
            .error_for_status()
            .map_err(|_| AppError::Unauthorized)?
            .json::<GoogleTokenResponse>()
            .await
            .map_err(|_| AppError::Internal("Google authorization failed".into()))?;
        self.verify_id_token(&response.id_token, expected_nonce)
            .await
    }
}

#[derive(Deserialize)]
struct GoogleTokenResponse {
    id_token: String,
}

#[derive(Deserialize)]
struct GoogleJwks {
    keys: Vec<GoogleJwk>,
}

#[derive(Deserialize)]
struct GoogleJwk {
    kid: Option<String>,
    n: String,
    e: String,
}

#[derive(Deserialize)]
struct GoogleClaims {
    iss: String,
    sub: String,
    aud: String,
    exp: usize,
    nonce: String,
    email: String,
    email_verified: bool,
    name: Option<String>,
}

fn normalize_display_name(name: String) -> Option<String> {
    let name = name.trim();
    (!name.is_empty() && name.chars().count() <= 256).then(|| name.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::GoogleConfig,
        sessions::{
            model::{GoogleIdentity, GoogleLoginState, UserRecord, UserRole},
            repository::SessionRepository,
        },
    };
    use std::sync::Mutex;

    #[derive(Default)]
    struct FakeRepository {
        state: Mutex<Option<(Vec<u8>, GoogleLoginState)>>,
        completed: Mutex<Vec<GoogleIdentity>>,
    }

    #[async_trait]
    impl SessionRepository for FakeRepository {
        async fn create_anonymous(
            &self,
            _token_hash: &[u8],
            _expires_at: OffsetDateTime,
        ) -> Result<Uuid, AppError> {
            Err(AppError::Internal("unused test repository method".into()))
        }

        async fn find_active_anonymous(&self, _token_hash: &[u8]) -> Result<Uuid, AppError> {
            Err(AppError::Internal("unused test repository method".into()))
        }

        async fn find_active_auth_session(
            &self,
            _token_hash: &[u8],
        ) -> Result<Option<(Uuid, Uuid)>, AppError> {
            Err(AppError::Internal("unused test repository method".into()))
        }

        async fn find_user_by_email(&self, _email: &str) -> Result<Option<UserRecord>, AppError> {
            Err(AppError::Internal("unused test repository method".into()))
        }

        async fn find_user_by_id(&self, _user_id: Uuid) -> Result<UserRecord, AppError> {
            Err(AppError::Internal("unused test repository method".into()))
        }

        async fn list_users(&self) -> Result<Vec<UserRecord>, AppError> {
            Err(AppError::Internal("unused test repository method".into()))
        }

        async fn create_google_login_state(
            &self,
            state_hash: &[u8],
            nonce: &str,
            pkce_verifier: &str,
            _expires_at: OffsetDateTime,
            anonymous_session_id: Option<Uuid>,
        ) -> Result<(), AppError> {
            *self.state.lock().unwrap() = Some((
                state_hash.to_vec(),
                GoogleLoginState {
                    nonce: nonce.into(),
                    pkce_verifier: pkce_verifier.into(),
                    anonymous_session_id,
                },
            ));
            Ok(())
        }

        async fn consume_google_login_state(
            &self,
            state_hash: &[u8],
        ) -> Result<Option<GoogleLoginState>, AppError> {
            let mut state = self.state.lock().unwrap();
            if state
                .as_ref()
                .is_some_and(|(stored_hash, _)| stored_hash == state_hash)
            {
                return Ok(state.take().map(|(_, state)| state));
            }
            Ok(None)
        }

        async fn complete_google_login(
            &self,
            identity: &GoogleIdentity,
            _auth_token_hash: &[u8],
            _expires_at: OffsetDateTime,
            _transfer_session_id: Option<Uuid>,
        ) -> Result<UserRecord, AppError> {
            self.completed.lock().unwrap().push(identity.clone());
            Ok(UserRecord {
                id: Uuid::new_v4(),
                email: identity.email.clone(),
                password_hash: None,
                display_name: identity.display_name.clone(),
                role: UserRole::User,
                created_at: OffsetDateTime::now_utc(),
            })
        }

        async fn create_account(
            &self,
            _email: &str,
            _password_hash: &str,
            _display_name: Option<&str>,
            _auth_token_hash: &[u8],
            _expires_at: OffsetDateTime,
            _transfer_session_id: Option<Uuid>,
        ) -> Result<UserRecord, AppError> {
            Err(AppError::Internal("unused test repository method".into()))
        }

        async fn create_auth_session(
            &self,
            _user_id: Uuid,
            _token_hash: &[u8],
            _expires_at: OffsetDateTime,
        ) -> Result<Uuid, AppError> {
            Err(AppError::Internal("unused test repository method".into()))
        }

        async fn revoke_auth_session(&self, _token_hash: &[u8]) -> Result<(), AppError> {
            Err(AppError::Internal("unused test repository method".into()))
        }

        async fn revoke_anonymous_session(&self, _token_hash: &[u8]) -> Result<(), AppError> {
            Err(AppError::Internal("unused test repository method".into()))
        }

        async fn seed_user(
            &self,
            _email: &str,
            _password_hash: &str,
        ) -> Result<UserRecord, AppError> {
            Err(AppError::Internal("unused test repository method".into()))
        }
    }

    struct FakeProvider {
        identity: GoogleIdentity,
        calls: Mutex<Vec<(String, String, String)>>,
    }

    #[async_trait]
    impl GoogleOidcProvider for FakeProvider {
        async fn exchange_code(
            &self,
            code: &str,
            code_verifier: &str,
            expected_nonce: &str,
        ) -> Result<GoogleIdentity, AppError> {
            self.calls.lock().unwrap().push((
                code.into(),
                code_verifier.into(),
                expected_nonce.into(),
            ));
            Ok(self.identity.clone())
        }
    }

    fn google_config() -> GoogleConfig {
        GoogleConfig {
            client_id: "client-id".into(),
            client_secret: "client-secret".into(),
            redirect_uri: "http://localhost:18732/api/v1/auth/google/callback".into(),
        }
    }

    #[test]
    fn uses_the_rfc_pkce_s256_challenge() {
        assert_eq!(
            pkce_challenge("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
        );
    }

    #[test]
    fn compares_state_without_accepting_a_different_value() {
        assert!(constant_time_equal(b"state", b"state"));
        assert!(!constant_time_equal(b"state", b"other"));
    }

    #[test]
    fn keeps_only_non_empty_bounded_display_names() {
        assert_eq!(
            normalize_display_name("  Ada Lovelace ".into()).as_deref(),
            Some("Ada Lovelace")
        );
        assert!(normalize_display_name(" ".into()).is_none());
    }

    #[tokio::test]
    async fn creates_a_google_authorization_request_with_server_state() {
        let repository = Arc::new(FakeRepository::default());
        let provider = Arc::new(FakeProvider {
            identity: GoogleIdentity {
                subject: "google-subject".into(),
                email: "ada@example.com".into(),
                email_verified: true,
                display_name: Some("Ada Lovelace".into()),
            },
            calls: Mutex::new(Vec::new()),
        });
        let service = GoogleAuthService::new_with_provider(
            repository.clone(),
            Duration::hours(1),
            google_config(),
            provider,
        );

        let authorization = service.start(Some(Uuid::new_v4())).await.unwrap();
        let params: std::collections::HashMap<_, _> = Url::parse(&authorization.url)
            .unwrap()
            .query_pairs()
            .into_owned()
            .collect();
        let stored = repository.state.lock().unwrap().clone().unwrap().1;

        assert_eq!(params["client_id"], "client-id");
        assert_eq!(params["redirect_uri"], google_config().redirect_uri);
        assert_eq!(params["state"], authorization.state);
        assert_eq!(params["nonce"], stored.nonce);
        assert_eq!(
            params["code_challenge"],
            pkce_challenge(&stored.pkce_verifier)
        );
        assert_eq!(params["code_challenge_method"], "S256");
    }

    #[tokio::test]
    async fn validates_the_cookie_bound_state_before_completing_login() {
        let repository = Arc::new(FakeRepository::default());
        let provider = Arc::new(FakeProvider {
            identity: GoogleIdentity {
                subject: "google-subject".into(),
                email: "ADA@example.com".into(),
                email_verified: true,
                display_name: Some("Ada Lovelace".into()),
            },
            calls: Mutex::new(Vec::new()),
        });
        let service = GoogleAuthService::new_with_provider(
            repository.clone(),
            Duration::hours(1),
            google_config(),
            provider.clone(),
        );
        let authorization = service.start(None).await.unwrap();
        let stored = repository.state.lock().unwrap().clone().unwrap().1;

        assert!(matches!(
            service
                .callback(&authorization.state, Some("wrong-state"), "code")
                .await,
            Err(AppError::Unauthorized)
        ));
        let session = service
            .callback(&authorization.state, Some(&authorization.state), "code")
            .await
            .unwrap();

        assert_eq!(session.account.email, "ada@example.com");
        assert_eq!(
            session.account.display_name.as_deref(),
            Some("Ada Lovelace")
        );
        assert_eq!(repository.completed.lock().unwrap().len(), 1);
        assert_eq!(
            provider.calls.lock().unwrap()[0],
            ("code".into(), stored.pkce_verifier, stored.nonce)
        );
    }
}
