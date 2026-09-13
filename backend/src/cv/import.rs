use std::sync::Arc;

use serde::Deserialize;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use super::{
    linkedin::normalize_profile,
    model::{CvData, CvSessionResponse, PendingCvImportResponse},
    repository::CvRepository,
    template,
};
use crate::{error::AppError, sessions::model::Principal};

const PENDING_IMPORT_TTL: Duration = Duration::minutes(15);

#[derive(Clone, Debug, Deserialize)]
pub struct ApplyPendingImportRequest {
    #[serde(alias = "version", alias = "base_version")]
    pub expected_version: i64,
}

pub struct CvImportService {
    repository: Arc<dyn CvRepository>,
}

impl CvImportService {
    pub fn new(repository: Arc<dyn CvRepository>) -> Self {
        Self { repository }
    }

    pub async fn create_pending(
        &self,
        user_id: Uuid,
        profile: &serde_json::Value,
    ) -> Result<PendingCvImportResponse, AppError> {
        let data = normalize_profile(profile)?;
        validate_normalized_data(&data)?;
        self.repository
            .create_pending_import(
                user_id,
                &data,
                OffsetDateTime::now_utc() + PENDING_IMPORT_TTL,
            )
            .await
    }

    pub async fn list_pending(
        &self,
        principal: &Principal,
    ) -> Result<Vec<PendingCvImportResponse>, AppError> {
        let user_id = require_user(principal)?;
        self.repository.list_pending_imports(user_id).await
    }

    pub async fn apply_pending(
        &self,
        principal: &Principal,
        import_id: Uuid,
        expected_version: i64,
    ) -> Result<CvSessionResponse, AppError> {
        let user_id = require_user(principal)?;
        self.repository
            .apply_pending_import(user_id, import_id, expected_version)
            .await
    }

    pub async fn delete_pending(
        &self,
        principal: &Principal,
        import_id: Uuid,
    ) -> Result<(), AppError> {
        let user_id = require_user(principal)?;
        self.repository
            .delete_pending_import(user_id, import_id)
            .await
    }
}

fn require_user(principal: &Principal) -> Result<Uuid, AppError> {
    principal.user_id().ok_or(AppError::Unauthorized)
}

fn validate_normalized_data(data: &CvData) -> Result<(), AppError> {
    let value = serde_json::to_value(data)
        .map_err(|error| AppError::Internal(format!("failed to serialize CV import: {error}")))?;
    template::parse_cv_data(&value).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn normalized_imports_are_valid_render_data() {
        validate_normalized_data(&serde_json::from_value(serde_json::json!({})).unwrap()).unwrap();
    }
}
