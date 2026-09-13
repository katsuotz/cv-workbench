use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub enum Principal {
    Anonymous {
        session_id: Uuid,
    },
    User {
        user_id: Uuid,
        auth_session_id: Uuid,
    },
}

impl Principal {
    pub fn user_id(&self) -> Option<Uuid> {
        match self {
            Self::Anonymous { .. } => None,
            Self::User { user_id, .. } => Some(*user_id),
        }
    }

    pub fn anonymous_session_id(&self) -> Option<Uuid> {
        match self {
            Self::Anonymous { session_id } => Some(*session_id),
            Self::User { .. } => None,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct AnonymousSessionResponse {
    pub session_id: Uuid,
    pub token: String,
    pub expires_at: OffsetDateTime,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub name: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct AccountResponse {
    pub id: Uuid,
    pub email: String,
    #[serde(rename = "name")]
    pub display_name: Option<String>,
    pub created_at: OffsetDateTime,
}

#[derive(Clone, Debug)]
pub struct UserRecord {
    pub id: Uuid,
    pub email: String,
    pub password_hash: Option<String>,
    pub display_name: Option<String>,
    pub created_at: OffsetDateTime,
}

#[derive(Clone, Debug)]
pub struct GoogleIdentity {
    pub subject: String,
    pub email: String,
    pub email_verified: bool,
    pub display_name: Option<String>,
}

#[derive(Clone, Debug)]
pub struct GoogleLoginState {
    pub nonce: String,
    pub pkce_verifier: String,
    pub anonymous_session_id: Option<Uuid>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkedInIntent {
    Login,
    Import,
}

impl LinkedInIntent {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Login => "login",
            Self::Import => "import",
        }
    }
}

#[derive(Clone, Debug)]
pub struct LinkedInIdentity {
    pub subject: String,
    pub email: Option<String>,
    pub email_verified: bool,
    pub display_name: Option<String>,
}

#[derive(Clone, Debug)]
pub struct LinkedInLoginState {
    pub intent: LinkedInIntent,
    pub nonce: String,
    pub pkce_verifier: String,
    pub anonymous_session_id: Option<Uuid>,
    pub authenticated_user_id: Option<Uuid>,
}

#[derive(Clone, Debug)]
pub struct AuthSessionResponse {
    pub account: AccountResponse,
    pub token: String,
    pub expires_at: OffsetDateTime,
}
