use std::sync::Arc;

use async_trait::async_trait;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use rand::RngCore;
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use time::{Duration, OffsetDateTime};
use url::Url;
use uuid::Uuid;

use super::{
    model::{
        AuthSessionResponse, LinkedInIdentity, LinkedInIntent, LinkedInLoginState, UserRecord,
    },
    repository::SessionRepository,
    service::{account_response, normalize_display_name, normalize_email, token_hash},
};
use crate::{config::LinkedInConfig, error::AppError};

const LINKEDIN_AUTHORIZATION_ENDPOINT: &str = "https://www.linkedin.com/oauth/v2/authorization";
const LINKEDIN_TOKEN_ENDPOINT: &str = "https://www.linkedin.com/oauth/v2/accessToken";
const LINKEDIN_JWKS_ENDPOINT: &str = "https://www.linkedin.com/oauth/openid/jwks";
const LINKEDIN_ISSUER: &str = "https://www.linkedin.com";
const MAX_PROVIDER_PROFILE_BYTES: usize = 4 * 1_048_576;
pub const LINKEDIN_STATE_COOKIE: &str = "lr_linkedin_state";
pub const LINKEDIN_STATE_TTL: Duration = Duration::minutes(10);

#[derive(Clone, Debug)]
pub struct LinkedInAuthorization {
    pub url: String,
    pub state: String,
}

#[derive(Clone, Debug)]
pub struct LinkedInProviderResponse {
    pub identity: LinkedInIdentity,
    pub profile: Option<Value>,
}

#[async_trait]
pub trait LinkedInProvider: Send + Sync {
    async fn exchange_code(
        &self,
        code: &str,
        code_verifier: &str,
        expected_nonce: &str,
        intent: LinkedInIntent,
    ) -> Result<LinkedInProviderResponse, AppError>;
}

pub struct LinkedInAuthService {
    repository: Arc<dyn SessionRepository>,
    provider: Option<Arc<dyn LinkedInProvider>>,
    config: Option<LinkedInConfig>,
    ttl: Duration,
}

#[derive(Clone, Debug)]
pub struct LinkedInCallback {
    pub intent: LinkedInIntent,
    pub user: UserRecord,
    pub profile: Option<Value>,
    pub auth_session: Option<AuthSessionResponse>,
}

impl LinkedInAuthService {
    pub fn new(
        repository: Arc<dyn SessionRepository>,
        ttl: Duration,
        config: Option<LinkedInConfig>,
    ) -> Self {
        let provider = config.as_ref().map(|config| {
            Arc::new(LinkedInHttpProvider::new(config.clone())) as Arc<dyn LinkedInProvider>
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
        config: LinkedInConfig,
        provider: Arc<dyn LinkedInProvider>,
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
        intent: LinkedInIntent,
        anonymous_session_id: Option<Uuid>,
        authenticated_user_id: Option<Uuid>,
    ) -> Result<LinkedInAuthorization, AppError> {
        let config = self.config.as_ref().ok_or_else(|| {
            AppError::BadRequest("LinkedIn login is not configured on this server".into())
        })?;
        let scopes = match intent {
            LinkedInIntent::Login => "openid profile email".to_owned(),
            LinkedInIntent::Import => {
                let scopes = config
                    .import
                    .as_ref()
                    .map(|import| normalize_scopes(&import.scopes))
                    .filter(|scopes| !scopes.is_empty())
                    .ok_or_else(|| {
                        AppError::BadRequest(
                            "LinkedIn full-profile import is not approved or configured on this server"
                                .into(),
                        )
                    })?;
                identity_scopes(&scopes)
            }
        };
        let state = random_token();
        let nonce = random_token();
        let pkce_verifier = random_token();
        let login_state = LinkedInLoginState {
            intent,
            nonce: nonce.clone(),
            pkce_verifier: pkce_verifier.clone(),
            anonymous_session_id,
            authenticated_user_id,
        };
        self.repository
            .create_linkedin_login_state(
                &token_hash(&state),
                login_state.intent,
                &login_state.nonce,
                &login_state.pkce_verifier,
                OffsetDateTime::now_utc() + LINKEDIN_STATE_TTL,
                login_state.anonymous_session_id,
                login_state.authenticated_user_id,
            )
            .await?;

        let mut url = Url::parse(LINKEDIN_AUTHORIZATION_ENDPOINT)
            .map_err(|error| AppError::Internal(error.to_string()))?;
        url.query_pairs_mut()
            .append_pair("client_id", &config.client_id)
            .append_pair("redirect_uri", &config.redirect_uri)
            .append_pair("response_type", "code")
            .append_pair("scope", &scopes)
            .append_pair("state", &state)
            .append_pair("nonce", &nonce)
            .append_pair("code_challenge", &pkce_challenge(&pkce_verifier))
            .append_pair("code_challenge_method", "S256");
        Ok(LinkedInAuthorization {
            url: url.into(),
            state,
        })
    }

    pub async fn callback(
        &self,
        state: &str,
        state_cookie: Option<&str>,
        code: &str,
    ) -> Result<LinkedInCallback, AppError> {
        validate_state_cookie(state, state_cookie)?;
        let login_state = self
            .repository
            .consume_linkedin_login_state(&token_hash(state))
            .await?
            .ok_or(AppError::Unauthorized)?;
        let provider = self.provider.as_ref().ok_or_else(|| {
            AppError::BadRequest("LinkedIn login is not configured on this server".into())
        })?;
        let response = provider
            .exchange_code(
                code,
                &login_state.pkce_verifier,
                &login_state.nonce,
                login_state.intent,
            )
            .await?;
        let identity = normalize_identity(
            response.identity,
            login_state.intent,
            login_state.authenticated_user_id,
        )?;
        let token = (login_state.intent == LinkedInIntent::Login
            || login_state.authenticated_user_id.is_none())
        .then(random_token);
        let token_hash = token.as_deref().map(token_hash).unwrap_or([0_u8; 32]);
        let expires_at = OffsetDateTime::now_utc() + self.ttl;
        let user = match login_state.intent {
            LinkedInIntent::Login => {
                self.repository
                    .complete_linkedin_login(
                        &identity,
                        &token_hash,
                        expires_at,
                        login_state.anonymous_session_id,
                    )
                    .await?
            }
            LinkedInIntent::Import => {
                if response.profile.is_none() {
                    return Err(AppError::BadRequest(
                        "LinkedIn full-profile import returned no approved profile data".into(),
                    ));
                }
                self.repository
                    .complete_linkedin_import(
                        &identity,
                        login_state.authenticated_user_id,
                        &token_hash,
                        expires_at,
                        login_state.anonymous_session_id,
                    )
                    .await?
            }
        };
        let auth_session = token.map(|token| AuthSessionResponse {
            account: account_response(&user),
            token,
            expires_at,
        });
        Ok(LinkedInCallback {
            intent: login_state.intent,
            user,
            profile: response.profile,
            auth_session,
        })
    }

    pub async fn cancel(&self, state: &str, state_cookie: Option<&str>) -> Result<(), AppError> {
        validate_state_cookie(state, state_cookie)?;
        self.repository
            .consume_linkedin_login_state(&token_hash(state))
            .await?
            .ok_or(AppError::Unauthorized)?;
        Ok(())
    }
}

fn normalize_identity(
    mut identity: LinkedInIdentity,
    intent: LinkedInIntent,
    authenticated_user_id: Option<Uuid>,
) -> Result<LinkedInIdentity, AppError> {
    identity.subject = identity.subject.trim().to_owned();
    if identity.subject.is_empty() || identity.subject.chars().count() > 256 {
        return Err(AppError::Unauthorized);
    }
    identity.display_name = identity
        .display_name
        .as_deref()
        .map(|name| normalize_display_name(Some(name)))
        .transpose()?
        .flatten();
    if let Some(email) = identity.email.as_deref() {
        identity.email = Some(normalize_email(email)?);
    }
    if intent == LinkedInIntent::Login
        || authenticated_user_id.is_none()
        || identity.email.is_some()
    {
        if !identity.email_verified {
            return Err(AppError::Unauthorized);
        }
    }
    if (intent == LinkedInIntent::Login || authenticated_user_id.is_none())
        && identity.email.is_none()
    {
        return Err(AppError::Unauthorized);
    }
    Ok(identity)
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
        "{LINKEDIN_STATE_COOKIE}={value}; Path=/api/v1/auth/linkedin; HttpOnly; SameSite=Lax; Max-Age={max_age}{secure_suffix}"
    ))
    .map_err(|error| AppError::Internal(error.to_string()))
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

fn random_token() -> String {
    let mut bytes = [0_u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

pub fn pkce_challenge(verifier: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}

fn normalize_scopes(scopes: &str) -> String {
    scopes
        .split(|character: char| character.is_ascii_whitespace() || character == ',')
        .filter(|scope| !scope.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn identity_scopes(scopes: &str) -> String {
    let mut normalized = scopes.to_owned();
    for scope in ["openid", "profile", "email"] {
        if !normalized.split_whitespace().any(|value| value == scope) {
            normalized.push(' ');
            normalized.push_str(scope);
        }
    }
    normalized
}

struct LinkedInHttpProvider {
    client: Client,
    config: LinkedInConfig,
}

impl LinkedInHttpProvider {
    fn new(config: LinkedInConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    async fn verify_id_token(
        &self,
        id_token: &str,
        expected_nonce: &str,
    ) -> Result<LinkedInIdentity, AppError> {
        let header = decode_header(id_token).map_err(|_| AppError::Unauthorized)?;
        if header.alg != Algorithm::RS256 {
            return Err(AppError::Unauthorized);
        }
        let kid = header.kid.ok_or(AppError::Unauthorized)?;
        let jwks = self
            .client
            .get(LINKEDIN_JWKS_ENDPOINT)
            .send()
            .await
            .map_err(|_| AppError::Internal("LinkedIn identity verification failed".into()))?
            .error_for_status()
            .map_err(|_| AppError::Internal("LinkedIn identity verification failed".into()))?
            .json::<LinkedInJwks>()
            .await
            .map_err(|_| AppError::Internal("LinkedIn identity verification failed".into()))?;
        let key = jwks
            .keys
            .into_iter()
            .find(|key| key.kid.as_deref() == Some(kid.as_str()))
            .ok_or(AppError::Unauthorized)?;
        let decoding_key =
            DecodingKey::from_rsa_components(&key.n, &key.e).map_err(|_| AppError::Unauthorized)?;
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&[LINKEDIN_ISSUER]);
        validation.set_audience(std::slice::from_ref(&self.config.client_id));
        let claims = decode::<LinkedInClaims>(id_token, &decoding_key, &validation)
            .map_err(|_| AppError::Unauthorized)?
            .claims;
        if claims.iss != LINKEDIN_ISSUER
            || claims.aud != self.config.client_id
            || claims.exp <= OffsetDateTime::now_utc().unix_timestamp() as usize
            || claims.nonce != expected_nonce
            || claims.sub.trim().is_empty()
        {
            return Err(AppError::Unauthorized);
        }
        Ok(LinkedInIdentity {
            subject: claims.sub,
            email: claims.email,
            email_verified: claims.email_verified.unwrap_or(false),
            display_name: claims.name.or_else(|| {
                [claims.given_name, claims.family_name]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>()
                    .join(" ")
                    .into()
            }),
        })
    }
}

#[async_trait]
impl LinkedInProvider for LinkedInHttpProvider {
    async fn exchange_code(
        &self,
        code: &str,
        code_verifier: &str,
        expected_nonce: &str,
        intent: LinkedInIntent,
    ) -> Result<LinkedInProviderResponse, AppError> {
        let token_response = self
            .client
            .post(LINKEDIN_TOKEN_ENDPOINT)
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
            .map_err(|_| AppError::Internal("LinkedIn authorization failed".into()))?
            .error_for_status()
            .map_err(|_| AppError::Unauthorized)?
            .json::<LinkedInTokenResponse>()
            .await
            .map_err(|_| AppError::Internal("LinkedIn authorization failed".into()))?;
        let identity = self
            .verify_id_token(
                token_response
                    .id_token
                    .as_deref()
                    .ok_or(AppError::Unauthorized)?,
                expected_nonce,
            )
            .await?;
        let profile = if intent == LinkedInIntent::Import {
            let endpoint = self
                .config
                .import
                .as_ref()
                .map(|import| import.profile_endpoint.as_str())
                .ok_or_else(|| {
                    AppError::BadRequest(
                        "LinkedIn full-profile import is not approved or configured on this server"
                            .into(),
                    )
                })?;
            let response = self
                .client
                .get(endpoint)
                .bearer_auth(&token_response.access_token)
                .send()
                .await
                .map_err(|_| AppError::Internal("LinkedIn profile import failed".into()))?
                .error_for_status()
                .map_err(|_| {
                    AppError::BadRequest(
                        "LinkedIn profile import was not approved by the provider".into(),
                    )
                })?;
            let mut bytes = Vec::new();
            let mut response = response;
            while let Some(chunk) = response
                .chunk()
                .await
                .map_err(|_| AppError::Internal("LinkedIn profile import failed".into()))?
            {
                if bytes.len() + chunk.len() > MAX_PROVIDER_PROFILE_BYTES {
                    return Err(AppError::BadRequest(
                        "LinkedIn profile import response is too large".into(),
                    ));
                }
                bytes.extend_from_slice(&chunk);
            }
            Some(serde_json::from_slice(&bytes).map_err(|_| {
                AppError::BadRequest("LinkedIn profile import returned invalid data".into())
            })?)
        } else {
            None
        };
        Ok(LinkedInProviderResponse { identity, profile })
    }
}

#[derive(Deserialize)]
struct LinkedInTokenResponse {
    access_token: String,
    id_token: Option<String>,
}

#[derive(Deserialize)]
struct LinkedInJwks {
    keys: Vec<LinkedInJwk>,
}

#[derive(Deserialize)]
struct LinkedInJwk {
    kid: Option<String>,
    n: String,
    e: String,
}

#[derive(Deserialize)]
struct LinkedInClaims {
    iss: String,
    sub: String,
    aud: String,
    exp: usize,
    nonce: String,
    email: Option<String>,
    email_verified: Option<bool>,
    name: Option<String>,
    given_name: Option<String>,
    family_name: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_s256_and_normalizes_approved_scopes() {
        assert_eq!(
            pkce_challenge("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
        );
        assert_eq!(
            normalize_scopes("r_fullprofile, r_emailaddress"),
            "r_fullprofile r_emailaddress"
        );
        assert_eq!(
            identity_scopes("r_fullprofile r_emailaddress"),
            "r_fullprofile r_emailaddress openid profile email"
        );
    }

    #[test]
    fn requires_verified_email_for_new_or_login_accounts() {
        let error = normalize_identity(
            LinkedInIdentity {
                subject: "subject".into(),
                email: None,
                email_verified: false,
                display_name: None,
            },
            LinkedInIntent::Login,
            None,
        )
        .unwrap_err();
        assert!(matches!(error, AppError::Unauthorized));
    }

    #[test]
    fn allows_authenticated_import_to_link_without_relying_on_email_matching() {
        let identity = normalize_identity(
            LinkedInIdentity {
                subject: "subject".into(),
                email: None,
                email_verified: false,
                display_name: None,
            },
            LinkedInIntent::Import,
            Some(Uuid::new_v4()),
        )
        .unwrap();
        assert_eq!(identity.subject, "subject");
    }
}
