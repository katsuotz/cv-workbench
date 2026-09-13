use axum::{
    Json,
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header},
    response::Response,
};

use super::import::ApplyPendingImportRequest;
use super::model::{
    CvSessionResponse, RenderCvRequest, RenderCvResponse, SaveCvSessionRequest, TemplateCatalogItem,
};
use crate::sessions::routes::validate_origin;
use crate::{AppState, error::AppError};

pub async fn get_session(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Result<Json<CvSessionResponse>, AppError> {
    let principal = state.sessions.authenticate(&headers).await?;
    state
        .cv
        .get(&principal)
        .await?
        .map(Json)
        .ok_or(AppError::NotFound)
}

pub async fn save_session(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(request): Json<SaveCvSessionRequest>,
) -> Result<(StatusCode, Json<CvSessionResponse>), AppError> {
    validate_origin(&headers, &state.config, true)?;
    let principal = state.sessions.authenticate(&headers).await?;
    let created = request.version == 0 && state.cv.get(&principal).await?.is_none();
    let session = state.cv.save(&principal, request).await?;
    Ok((
        if created {
            StatusCode::CREATED
        } else {
            StatusCode::OK
        },
        Json(session),
    ))
}

pub async fn list_templates(
    State(state): State<crate::AppState>,
) -> Result<Json<Vec<TemplateCatalogItem>>, AppError> {
    Ok(Json(state.cv_templates.catalog().await?))
}

pub async fn template_preview(
    State(state): State<crate::AppState>,
    Path(template_id): Path<String>,
) -> Result<Response, AppError> {
    let (bytes, media_type) = state.cv_templates.preview(&template_id).await?;
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, media_type)
        .header(header::CACHE_CONTROL, "public, max-age=3600")
        .body(Body::from(bytes))
        .map_err(|error| AppError::Internal(error.to_string()))
}

pub async fn render_cv(
    State(state): State<crate::AppState>,
    headers: HeaderMap,
    Json(request): Json<RenderCvRequest>,
) -> Result<Json<RenderCvResponse>, AppError> {
    validate_origin(&headers, &state.config, true)?;
    state.sessions.authenticate(&headers).await?;
    Ok(Json(state.cv_render.render(request).await?))
}

pub async fn get_pending_imports(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<super::model::PendingCvImportResponse>, AppError> {
    let principal = state.sessions.authenticate(&headers).await?;
    let pending = state
        .cv_import
        .list_pending(&principal)
        .await?
        .into_iter()
        .next()
        .ok_or(AppError::NotFound)?;
    Ok(Json(pending))
}

pub async fn apply_pending_import(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(import_id): Path<uuid::Uuid>,
    Json(request): Json<ApplyPendingImportRequest>,
) -> Result<Json<CvSessionResponse>, AppError> {
    validate_origin(&headers, &state.config, true)?;
    let principal = state.sessions.authenticate(&headers).await?;
    Ok(Json(
        state
            .cv_import
            .apply_pending(&principal, import_id, request.expected_version)
            .await?,
    ))
}

pub async fn delete_pending_import(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(import_id): Path<uuid::Uuid>,
) -> Result<StatusCode, AppError> {
    validate_origin(&headers, &state.config, true)?;
    let principal = state.sessions.authenticate(&headers).await?;
    state
        .cv_import
        .delete_pending(&principal, import_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_draft(
    state: State<crate::AppState>,
    headers: axum::http::HeaderMap,
) -> Result<Json<CvSessionResponse>, AppError> {
    get_session(state, headers).await
}

pub async fn save_draft(
    state: State<crate::AppState>,
    headers: axum::http::HeaderMap,
    request: Json<SaveCvSessionRequest>,
) -> Result<(StatusCode, Json<CvSessionResponse>), AppError> {
    save_session(state, headers, request).await
}
