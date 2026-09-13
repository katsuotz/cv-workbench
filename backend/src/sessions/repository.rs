use async_trait::async_trait;
use sqlx::{PgPool, Row};
use time::OffsetDateTime;
use uuid::Uuid;

use super::model::{GoogleIdentity, GoogleLoginState, UserRecord};
use crate::error::AppError;

#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn create_anonymous(
        &self,
        token_hash: &[u8],
        expires_at: OffsetDateTime,
    ) -> Result<Uuid, AppError>;

    async fn find_active_anonymous(&self, token_hash: &[u8]) -> Result<Uuid, AppError>;

    async fn find_active_auth_session(
        &self,
        token_hash: &[u8],
    ) -> Result<Option<(Uuid, Uuid)>, AppError>;

    async fn find_user_by_email(&self, email: &str) -> Result<Option<UserRecord>, AppError>;
    async fn find_user_by_id(&self, user_id: Uuid) -> Result<UserRecord, AppError>;

    async fn create_google_login_state(
        &self,
        state_hash: &[u8],
        nonce: &str,
        pkce_verifier: &str,
        expires_at: OffsetDateTime,
        anonymous_session_id: Option<Uuid>,
    ) -> Result<(), AppError>;

    async fn consume_google_login_state(
        &self,
        state_hash: &[u8],
    ) -> Result<Option<GoogleLoginState>, AppError>;

    async fn complete_google_login(
        &self,
        identity: &GoogleIdentity,
        auth_token_hash: &[u8],
        expires_at: OffsetDateTime,
        transfer_session_id: Option<Uuid>,
    ) -> Result<UserRecord, AppError>;

    async fn create_account(
        &self,
        email: &str,
        password_hash: &str,
        display_name: Option<&str>,
        auth_token_hash: &[u8],
        expires_at: OffsetDateTime,
        transfer_session_id: Option<Uuid>,
    ) -> Result<UserRecord, AppError>;

    async fn create_auth_session(
        &self,
        user_id: Uuid,
        token_hash: &[u8],
        expires_at: OffsetDateTime,
    ) -> Result<Uuid, AppError>;

    async fn revoke_auth_session(&self, token_hash: &[u8]) -> Result<(), AppError>;
    async fn revoke_anonymous_session(&self, token_hash: &[u8]) -> Result<(), AppError>;

    async fn seed_user(&self, email: &str, password_hash: &str) -> Result<UserRecord, AppError>;
}

pub struct PgSessionRepository {
    pool: PgPool,
}

impl PgSessionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SessionRepository for PgSessionRepository {
    async fn create_anonymous(
        &self,
        token_hash: &[u8],
        expires_at: OffsetDateTime,
    ) -> Result<Uuid, AppError> {
        Ok(sqlx::query_scalar(
            "INSERT INTO anonymous_sessions (token_hash, expires_at) VALUES ($1, $2) RETURNING id",
        )
        .bind(token_hash)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await?)
    }

    async fn find_active_anonymous(&self, token_hash: &[u8]) -> Result<Uuid, AppError> {
        sqlx::query_scalar(
            "SELECT id FROM anonymous_sessions WHERE token_hash = $1 AND revoked_at IS NULL AND expires_at > now()",
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(AppError::Unauthorized)
    }

    async fn find_active_auth_session(
        &self,
        token_hash: &[u8],
    ) -> Result<Option<(Uuid, Uuid)>, AppError> {
        Ok(sqlx::query_as(
            "SELECT id, user_id FROM auth_sessions WHERE token_hash = $1 AND revoked_at IS NULL AND expires_at > now()",
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await?)
    }

    async fn find_user_by_email(&self, email: &str) -> Result<Option<UserRecord>, AppError> {
        Ok(sqlx::query(
            "SELECT id, email, password_hash, display_name, created_at FROM users WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?
        .map(user_from_row)
        .transpose()?)
    }

    async fn find_user_by_id(&self, user_id: Uuid) -> Result<UserRecord, AppError> {
        sqlx::query(
            "SELECT id, email, password_hash, display_name, created_at FROM users WHERE id = $1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?
        .map(user_from_row)
        .transpose()?
        .ok_or(AppError::Unauthorized)
    }

    async fn create_google_login_state(
        &self,
        state_hash: &[u8],
        nonce: &str,
        pkce_verifier: &str,
        expires_at: OffsetDateTime,
        anonymous_session_id: Option<Uuid>,
    ) -> Result<(), AppError> {
        let mut transaction = self.pool.begin().await?;
        sqlx::query(
            "DELETE FROM google_login_states WHERE expires_at <= now() OR (anonymous_session_id IS NOT NULL AND anonymous_session_id = $1)",
        )
        .bind(anonymous_session_id)
        .execute(&mut *transaction)
        .await?;
        sqlx::query(
            "INSERT INTO google_login_states (state_hash, nonce, pkce_verifier, expires_at, anonymous_session_id) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(state_hash)
        .bind(nonce)
        .bind(pkce_verifier)
        .bind(expires_at)
        .bind(anonymous_session_id)
        .execute(&mut *transaction)
        .await?;
        transaction.commit().await?;
        Ok(())
    }

    async fn consume_google_login_state(
        &self,
        state_hash: &[u8],
    ) -> Result<Option<GoogleLoginState>, AppError> {
        sqlx::query(
            "DELETE FROM google_login_states WHERE state_hash = $1 AND expires_at > now() RETURNING nonce, pkce_verifier, anonymous_session_id",
        )
        .bind(state_hash)
        .fetch_optional(&self.pool)
        .await?
        .map(|row| {
            Ok(GoogleLoginState {
                nonce: row.try_get("nonce")?,
                pkce_verifier: row.try_get("pkce_verifier")?,
                anonymous_session_id: row.try_get("anonymous_session_id")?,
            })
        })
        .transpose()
    }

    async fn complete_google_login(
        &self,
        identity: &GoogleIdentity,
        auth_token_hash: &[u8],
        expires_at: OffsetDateTime,
        transfer_session_id: Option<Uuid>,
    ) -> Result<UserRecord, AppError> {
        let mut transaction = self.pool.begin().await?;
        if let Some(session_id) = transfer_session_id {
            let session_exists = sqlx::query_scalar::<_, Uuid>(
                "SELECT id FROM anonymous_sessions WHERE id = $1 AND revoked_at IS NULL AND expires_at > now() FOR UPDATE",
            )
            .bind(session_id)
            .fetch_optional(&mut *transaction)
            .await?
            .is_some();
            if !session_exists {
                return Err(AppError::Conflict(
                    "anonymous CV session is no longer available".into(),
                ));
            }
        }
        let existing_user_id = sqlx::query_scalar::<_, Uuid>(
            "SELECT user_id FROM oauth_identities WHERE provider = 'google' AND subject = $1 FOR UPDATE",
        )
        .bind(&identity.subject)
        .fetch_optional(&mut *transaction)
        .await?;
        let user = if let Some(user_id) = existing_user_id {
            let user_row = sqlx::query(
                "SELECT id, email, password_hash, display_name, created_at FROM users WHERE id = $1 FOR UPDATE",
            )
            .bind(user_id)
            .fetch_one(&mut *transaction)
            .await?;
            user_from_row(user_row)?
        } else {
            let user_row = sqlx::query(
                "INSERT INTO users (email, password_hash, display_name) VALUES ($1, NULL, $2) ON CONFLICT (email) DO UPDATE SET email = users.email RETURNING id, email, password_hash, display_name, created_at",
            )
            .bind(&identity.email)
            .bind(&identity.display_name)
            .fetch_one(&mut *transaction)
            .await?;
            let user = user_from_row(user_row)?;
            match sqlx::query(
                "INSERT INTO oauth_identities (provider, subject, user_id) VALUES ('google', $1, $2)",
            )
            .bind(&identity.subject)
            .bind(user.id)
            .execute(&mut *transaction)
            .await
            {
                Ok(_) => user,
                Err(error) if is_unique_violation(&error) => {
                    return Err(AppError::Conflict(
                        "Google account is already linked".into(),
                    ));
                }
                Err(error) => return Err(error.into()),
            }
        };

        if identity.display_name.is_some() {
            sqlx::query("UPDATE users SET display_name = COALESCE(display_name, $2) WHERE id = $1")
                .bind(user.id)
                .bind(&identity.display_name)
                .execute(&mut *transaction)
                .await?;
        }
        sqlx::query(
            "INSERT INTO auth_sessions (user_id, token_hash, expires_at) VALUES ($1, $2, $3)",
        )
        .bind(user.id)
        .bind(auth_token_hash)
        .bind(expires_at)
        .execute(&mut *transaction)
        .await?;
        if let Some(session_id) = transfer_session_id {
            sqlx::query(
                "UPDATE projects SET user_id = $1, session_id = NULL, updated_at = now() WHERE session_id = $2",
            )
            .bind(user.id)
            .bind(session_id)
            .execute(&mut *transaction)
            .await?;
            sqlx::query("UPDATE anonymous_sessions SET revoked_at = now() WHERE id = $1")
                .bind(session_id)
                .execute(&mut *transaction)
                .await?;
        }
        transaction.commit().await?;
        if identity.display_name.is_some() && user.display_name.is_none() {
            return self.find_user_by_id(user.id).await;
        }
        Ok(user)
    }

    async fn create_account(
        &self,
        email: &str,
        password_hash: &str,
        display_name: Option<&str>,
        auth_token_hash: &[u8],
        expires_at: OffsetDateTime,
        transfer_session_id: Option<Uuid>,
    ) -> Result<UserRecord, AppError> {
        let mut transaction = self.pool.begin().await?;
        let user = match sqlx::query(
            "INSERT INTO users (email, password_hash, display_name) VALUES ($1, $2, $3) RETURNING id, email, password_hash, display_name, created_at",
        )
        .bind(email)
        .bind(password_hash)
        .bind(display_name)
        .fetch_one(&mut *transaction)
        .await
        {
            Ok(row) => user_from_row(row)?,
            Err(error) if is_unique_violation(&error) => {
                return Err(AppError::Conflict("email is already registered".into()));
            }
            Err(error) => return Err(error.into()),
        };

        sqlx::query(
            "INSERT INTO auth_sessions (user_id, token_hash, expires_at) VALUES ($1, $2, $3)",
        )
        .bind(user.id)
        .bind(auth_token_hash)
        .bind(expires_at)
        .execute(&mut *transaction)
        .await?;

        if let Some(session_id) = transfer_session_id {
            sqlx::query(
                "UPDATE projects SET user_id = $1, session_id = NULL, updated_at = now() WHERE session_id = $2",
            )
            .bind(user.id)
            .bind(session_id)
            .execute(&mut *transaction)
            .await?;
            sqlx::query("UPDATE anonymous_sessions SET revoked_at = now() WHERE id = $1")
                .bind(session_id)
                .execute(&mut *transaction)
                .await?;
        }

        transaction.commit().await?;
        Ok(user)
    }

    async fn create_auth_session(
        &self,
        user_id: Uuid,
        token_hash: &[u8],
        expires_at: OffsetDateTime,
    ) -> Result<Uuid, AppError> {
        Ok(sqlx::query_scalar(
            "INSERT INTO auth_sessions (user_id, token_hash, expires_at) VALUES ($1, $2, $3) RETURNING id",
        )
        .bind(user_id)
        .bind(token_hash)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await?)
    }

    async fn revoke_auth_session(&self, token_hash: &[u8]) -> Result<(), AppError> {
        sqlx::query(
            "UPDATE auth_sessions SET revoked_at = now() WHERE token_hash = $1 AND revoked_at IS NULL",
        )
        .bind(token_hash)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn revoke_anonymous_session(&self, token_hash: &[u8]) -> Result<(), AppError> {
        sqlx::query(
            "UPDATE anonymous_sessions SET revoked_at = now() WHERE token_hash = $1 AND revoked_at IS NULL",
        )
        .bind(token_hash)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn seed_user(&self, email: &str, password_hash: &str) -> Result<UserRecord, AppError> {
        let row = sqlx::query(
            "INSERT INTO users (email, password_hash) VALUES ($1, $2) ON CONFLICT (email) DO UPDATE SET password_hash = EXCLUDED.password_hash RETURNING id, email, password_hash, display_name, created_at",
        )
        .bind(email)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await?;
        user_from_row(row)
    }
}

fn user_from_row(row: sqlx::postgres::PgRow) -> Result<UserRecord, AppError> {
    Ok(UserRecord {
        id: row.try_get("id")?,
        email: row.try_get("email")?,
        password_hash: row.try_get("password_hash")?,
        display_name: row.try_get("display_name")?,
        created_at: row.try_get("created_at")?,
    })
}

fn is_unique_violation(error: &sqlx::Error) -> bool {
    error
        .as_database_error()
        .and_then(|database_error| database_error.code())
        .is_some_and(|code| code == "23505")
}
