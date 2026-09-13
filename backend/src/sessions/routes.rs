use axum::{
    Json,
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::Redirect,
};
use serde::Deserialize;

use super::{
    google::{GOOGLE_STATE_COOKIE, clear_state_cookie_header, state_cookie_header},
    linkedin::{
        LINKEDIN_STATE_COOKIE, clear_state_cookie_header as clear_linkedin_state_cookie_header,
        state_cookie_header as linkedin_state_cookie_header,
    },
    model::{
        AccountResponse, AnonymousSessionResponse, LinkedInIntent, LoginRequest, Principal,
        RegisterRequest,
    },
    service::{ANONYMOUS_COOKIE, AUTH_COOKIE, bearer_token as parsed_bearer_token, cookie_value},
};
use crate::{AppState, config::Config, error::AppError};

pub async fn anonymous_session(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<(HeaderMap, Json<AnonymousSessionResponse>), AppError> {
    validate_origin(&headers, &state.config, true)?;
    let session = state.sessions.create_anonymous().await?;
    let mut response_headers = HeaderMap::new();
    response_headers.append(
        header::SET_COOKIE,
        cookie_header(
            ANONYMOUS_COOKIE,
            &session.token,
            state.config.session_ttl.as_secs(),
            state.config.cookie_secure,
        )?,
    );
    Ok((response_headers, Json(session)))
}

pub async fn register(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<RegisterRequest>,
) -> Result<(HeaderMap, Json<AccountResponse>), AppError> {
    validate_origin(&headers, &state.config, true)?;
    let transfer_session_id = match supplied_principal(&state, &headers).await? {
        Some(Principal::Anonymous { session_id }) => Some(session_id),
        Some(Principal::User { .. }) => {
            return Err(AppError::Conflict("an account is already signed in".into()));
        }
        None => None,
    };
    let session = state
        .sessions
        .register(
            &request.email,
            &request.password,
            request.name.as_deref(),
            transfer_session_id,
        )
        .await?;
    Ok((
        auth_cookie_headers(&state.config, &session.token),
        Json(session.account),
    ))
}

pub async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<LoginRequest>,
) -> Result<(HeaderMap, Json<AccountResponse>), AppError> {
    validate_origin(&headers, &state.config, true)?;
    let session = state
        .sessions
        .login(&request.email, &request.password)
        .await?;
    Ok((
        auth_cookie_headers(&state.config, &session.token),
        Json(session.account),
    ))
}

pub async fn google_start(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<(HeaderMap, Redirect), AppError> {
    validate_origin(&headers, &state.config, true)?;
    let transfer_session_id = match supplied_principal(&state, &headers).await? {
        Some(Principal::Anonymous { session_id }) => Some(session_id),
        Some(Principal::User { .. }) => {
            return Err(AppError::Conflict("an account is already signed in".into()));
        }
        None => None,
    };
    let authorization = match state.google.start(transfer_session_id).await {
        Ok(authorization) => authorization,
        Err(error) => {
            tracing::warn!(error = %error, "Google sign-in is unavailable");
            return Ok(google_error_redirect(
                &state,
                "Google sign-in is not available right now.",
                true,
            ));
        }
    };
    let mut response_headers = HeaderMap::new();
    response_headers.append(
        header::SET_COOKIE,
        state_cookie_header(&authorization.state, state.config.cookie_secure)?,
    );
    Ok((response_headers, Redirect::temporary(&authorization.url)))
}

#[derive(Debug, Deserialize)]
pub struct GoogleCallbackQuery {
    pub state: Option<String>,
    pub code: Option<String>,
    pub error: Option<String>,
}

pub async fn google_callback(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(query): axum::extract::Query<GoogleCallbackQuery>,
) -> Result<(HeaderMap, Redirect), AppError> {
    if query.error.is_some() {
        let state_consumed = state
            .google
            .cancel(
                query.state.as_deref().unwrap_or_default(),
                cookie_value(&headers, GOOGLE_STATE_COOKIE),
            )
            .await
            .is_ok();
        return Ok(google_error_redirect(
            &state,
            "Google sign-in was cancelled.",
            state_consumed,
        ));
    }
    let code = match query.code.as_deref().filter(|code| !code.is_empty()) {
        Some(code) => code,
        None => {
            return Ok(google_error_redirect(
                &state,
                "Google sign-in could not be completed.",
                false,
            ));
        }
    };
    let session = match state
        .google
        .callback(
            query.state.as_deref().unwrap_or_default(),
            cookie_value(&headers, GOOGLE_STATE_COOKIE),
            code,
        )
        .await
    {
        Ok(session) => session,
        Err(error) => {
            tracing::warn!(error = %error, "Google sign-in failed");
            return Ok(google_error_redirect(
                &state,
                "Google sign-in could not be completed.",
                true,
            ));
        }
    };
    let mut response_headers = auth_cookie_headers(&state.config, &session.token);
    response_headers.append(
        header::SET_COOKIE,
        clear_state_cookie_header(state.config.cookie_secure)?,
    );
    Ok((
        response_headers,
        frontend_redirect(&state, "success", None)?,
    ))
}

#[derive(Debug, Deserialize)]
pub struct LinkedInStartQuery {
    pub intent: Option<String>,
}

pub async fn linkedin_start(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(query): axum::extract::Query<LinkedInStartQuery>,
) -> Result<(HeaderMap, Redirect), AppError> {
    validate_origin(&headers, &state.config, true)?;
    let intent = match query.intent.as_deref() {
        Some("login") => LinkedInIntent::Login,
        Some("import") => LinkedInIntent::Import,
        _ => {
            return Err(AppError::BadRequest(
                "intent must be login or import".into(),
            ));
        }
    };
    let (anonymous_session_id, authenticated_user_id) =
        match supplied_principal(&state, &headers).await? {
            Some(Principal::Anonymous { session_id }) => (Some(session_id), None),
            Some(Principal::User { user_id, .. }) if intent == LinkedInIntent::Import => {
                (None, Some(user_id))
            }
            Some(Principal::User { .. }) => {
                return Err(AppError::Conflict("an account is already signed in".into()));
            }
            None => (None, None),
        };
    let authorization = state
        .linkedin
        .start(intent, anonymous_session_id, authenticated_user_id)
        .await?;
    let mut response_headers = HeaderMap::new();
    response_headers.append(
        header::SET_COOKIE,
        linkedin_state_cookie_header(&authorization.state, state.config.cookie_secure)?,
    );
    Ok((response_headers, Redirect::temporary(&authorization.url)))
}

#[derive(Debug, Deserialize)]
pub struct LinkedInCallbackQuery {
    pub state: Option<String>,
    pub code: Option<String>,
    pub error: Option<String>,
}

pub async fn linkedin_callback(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(query): axum::extract::Query<LinkedInCallbackQuery>,
) -> Result<(HeaderMap, Redirect), AppError> {
    if query.error.is_some() {
        let consumed = state
            .linkedin
            .cancel(
                query.state.as_deref().unwrap_or_default(),
                cookie_value(&headers, LINKEDIN_STATE_COOKIE),
            )
            .await
            .is_ok();
        return Ok(linkedin_error_redirect(
            &state,
            "LinkedIn sign-in was cancelled.",
            consumed,
        ));
    }
    let code = match query.code.as_deref().filter(|code| !code.is_empty()) {
        Some(code) => code,
        None => {
            return Ok(linkedin_error_redirect(
                &state,
                "LinkedIn sign-in could not be completed.",
                false,
            ));
        }
    };
    let callback = match state
        .linkedin
        .callback(
            query.state.as_deref().unwrap_or_default(),
            cookie_value(&headers, LINKEDIN_STATE_COOKIE),
            code,
        )
        .await
    {
        Ok(callback) => callback,
        Err(error) => {
            tracing::warn!(error = %error, "LinkedIn sign-in failed");
            return Ok(linkedin_error_redirect(
                &state,
                "LinkedIn sign-in could not be completed.",
                true,
            ));
        }
    };
    let mut response_headers = HeaderMap::new();
    if let Some(session) = callback.auth_session {
        response_headers.extend(auth_cookie_headers(&state.config, &session.token));
    }
    response_headers.append(
        header::SET_COOKIE,
        clear_linkedin_state_cookie_header(state.config.cookie_secure)?,
    );
    if callback.intent == LinkedInIntent::Import {
        let profile = callback.profile.as_ref().ok_or_else(|| {
            AppError::BadRequest(
                "LinkedIn full-profile import returned no approved profile data".into(),
            )
        })?;
        state
            .cv_import
            .create_pending(callback.user.id, profile)
            .await?;
        return Ok((response_headers, linkedin_import_redirect(&state)?));
    }
    Ok((
        response_headers,
        linkedin_frontend_redirect(&state, "success", None)?,
    ))
}

fn google_error_redirect(
    state: &AppState,
    message: &str,
    clear_state: bool,
) -> (HeaderMap, Redirect) {
    let mut response_headers = HeaderMap::new();
    if clear_state {
        if let Ok(cookie) = clear_state_cookie_header(state.config.cookie_secure) {
            response_headers.append(header::SET_COOKIE, cookie);
        }
    }
    let redirect = frontend_redirect(state, "error", Some(message))
        .unwrap_or_else(|_| Redirect::temporary("/app?auth=error"));
    (response_headers, redirect)
}

fn linkedin_error_redirect(
    state: &AppState,
    message: &str,
    clear_state: bool,
) -> (HeaderMap, Redirect) {
    let mut response_headers = HeaderMap::new();
    if clear_state {
        if let Ok(cookie) = clear_linkedin_state_cookie_header(state.config.cookie_secure) {
            response_headers.append(header::SET_COOKIE, cookie);
        }
    }
    let redirect = linkedin_frontend_redirect(state, "error", Some(message))
        .unwrap_or_else(|_| Redirect::temporary("/app?auth=error&provider=linkedin"));
    (response_headers, redirect)
}

fn linkedin_import_redirect(state: &AppState) -> Result<Redirect, AppError> {
    frontend_redirect_with_provider(state, "success", "linkedin", true, None)
}

fn linkedin_frontend_redirect(
    state: &AppState,
    result: &str,
    message: Option<&str>,
) -> Result<Redirect, AppError> {
    frontend_redirect_with_provider(state, result, "linkedin", false, message)
}

fn frontend_redirect(
    state: &AppState,
    result: &str,
    message: Option<&str>,
) -> Result<Redirect, AppError> {
    frontend_redirect_with_options(state, result, None, false, message)
}

fn frontend_redirect_with_provider(
    state: &AppState,
    result: &str,
    provider: &str,
    import_ready: bool,
    message: Option<&str>,
) -> Result<Redirect, AppError> {
    frontend_redirect_with_options(state, result, Some(provider), import_ready, message)
}

fn frontend_redirect_with_options(
    state: &AppState,
    result: &str,
    provider: Option<&str>,
    import_ready: bool,
    message: Option<&str>,
) -> Result<Redirect, AppError> {
    let origin = state
        .config
        .frontend_origin
        .to_str()
        .map_err(|error| AppError::Internal(error.to_string()))?;
    let mut url = url::Url::parse(origin).map_err(|error| AppError::Internal(error.to_string()))?;
    url.set_path("/app");
    url.set_query(None);
    {
        let mut query = url.query_pairs_mut();
        query.append_pair("auth", result);
        if let Some(provider) = provider {
            query.append_pair("provider", provider);
        }
        if import_ready {
            query.append_pair("import", "ready");
        }
        if let Some(message) = message {
            query.append_pair("message", message);
        }
    }
    Ok(Redirect::temporary(url.as_str()))
}

pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<(HeaderMap, StatusCode), AppError> {
    validate_origin(&headers, &state.config, true)?;
    state.sessions.logout(&headers).await?;
    Ok((clear_cookie_headers(&state.config), StatusCode::NO_CONTENT))
}

pub async fn me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<AccountResponse>, AppError> {
    let principal = state.sessions.authenticate(&headers).await?;
    Ok(Json(state.sessions.account(&principal).await?))
}

pub fn validate_origin(
    headers: &HeaderMap,
    config: &Config,
    state_changing: bool,
) -> Result<(), AppError> {
    if !state_changing {
        return Ok(());
    }
    if let Some(origin) = headers.get(header::ORIGIN) {
        if !config.origin_is_allowed(origin) {
            return Err(AppError::Forbidden);
        }
    }
    Ok(())
}

pub fn bearer_token(headers: &HeaderMap) -> Result<&str, AppError> {
    parsed_bearer_token(headers).ok_or(AppError::Unauthorized)
}

pub fn auth_cookie_headers(config: &Config, token: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    if let Ok(cookie) = cookie_header(
        AUTH_COOKIE,
        token,
        config.session_ttl.as_secs(),
        config.cookie_secure,
    ) {
        headers.append(header::SET_COOKIE, cookie);
    }
    headers
}

fn clear_cookie_headers(config: &Config) -> HeaderMap {
    let mut headers = HeaderMap::new();
    if let Ok(cookie) = cookie_header(AUTH_COOKIE, "", 0, config.cookie_secure) {
        headers.append(header::SET_COOKIE, cookie);
    }
    headers
}

fn cookie_header(
    name: &str,
    value: &str,
    max_age_seconds: u64,
    secure: bool,
) -> Result<HeaderValue, AppError> {
    let secure_suffix = if secure { "; Secure" } else { "" };
    let value = format!(
        "{name}={value}; Path=/; HttpOnly; SameSite=Lax; Max-Age={max_age_seconds}{secure_suffix}"
    );
    HeaderValue::try_from(value).map_err(|error| AppError::Internal(error.to_string()))
}

async fn supplied_principal(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<Option<Principal>, AppError> {
    let supplied = headers.contains_key(header::AUTHORIZATION)
        || cookie_value(headers, AUTH_COOKIE).is_some()
        || cookie_value(headers, ANONYMOUS_COOKIE).is_some();
    if supplied {
        Ok(Some(state.sessions.authenticate(headers).await?))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{net::SocketAddr, path::PathBuf, time::Duration};

    fn config() -> Config {
        Config {
            database_url: "postgres://localhost/test".into(),
            bind_addr: SocketAddr::from(([127, 0, 0, 1], 1)),
            frontend_origin: "http://localhost:5173".parse().unwrap(),
            compiler_enabled: false,
            compiler_path: PathBuf::from("xelatex"),
            compile_timeout: Duration::from_secs(30),
            session_ttl: Duration::from_secs(3600),
            cookie_secure: false,
            google: None,
            linkedin: None,
        }
    }

    #[test]
    fn rejects_cross_origin_state_changes() {
        let mut headers = HeaderMap::new();
        headers.insert(header::ORIGIN, "https://evil.example".parse().unwrap());
        assert!(matches!(
            validate_origin(&headers, &config(), true),
            Err(AppError::Forbidden)
        ));
    }

    #[test]
    fn allows_requests_without_origin_for_non_browser_clients() {
        let headers = HeaderMap::new();
        assert!(validate_origin(&headers, &config(), true).is_ok());
    }
}
