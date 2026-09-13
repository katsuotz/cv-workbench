pub mod compilation;
pub mod config;
pub mod cv;
pub mod documents;
pub mod error;
pub mod sessions;

use std::sync::Arc;

use axum::{
    Router,
    extract::DefaultBodyLimit,
    http::{Method, header},
    routing::{delete, get, post},
};
use sqlx::PgPool;
use tower_http::{cors::CorsLayer, timeout::TimeoutLayer, trace::TraceLayer};

use compilation::{CompilationService, repository::PgCompilationRepository};
use config::Config;
use cv::{
    CvImportService, CvRenderService, CvService, CvTemplateService, repository::PgCvRepository,
};
use documents::{DocumentService, repository::PgDocumentRepository};
use sessions::{
    SessionService, google::GoogleAuthService, linkedin::LinkedInAuthService,
    repository::PgSessionRepository,
};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<Config>,
    pub sessions: Arc<SessionService>,
    pub google: Arc<GoogleAuthService>,
    pub linkedin: Arc<LinkedInAuthService>,
    pub documents: Arc<DocumentService>,
    pub compilation: Arc<CompilationService>,
    pub cv: Arc<CvService>,
    pub cv_templates: Arc<CvTemplateService>,
    pub cv_render: Arc<CvRenderService>,
    pub cv_import: Arc<CvImportService>,
}

pub fn router(pool: PgPool, config: Config) -> Router {
    let config = Arc::new(config);
    let cors = CorsLayer::new()
        .allow_origin(config.frontend_origin.clone())
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([header::ACCEPT, header::AUTHORIZATION, header::CONTENT_TYPE])
        .allow_credentials(true);
    let session_repository = Arc::new(PgSessionRepository::new(pool.clone()));
    let sessions = Arc::new(SessionService::new(
        session_repository.clone(),
        time::Duration::seconds(config.session_ttl.as_secs() as i64),
    ));
    let google = Arc::new(GoogleAuthService::new(
        session_repository.clone(),
        time::Duration::seconds(config.session_ttl.as_secs() as i64),
        config.google.clone(),
    ));
    let linkedin = Arc::new(LinkedInAuthService::new(
        session_repository,
        time::Duration::seconds(config.session_ttl.as_secs() as i64),
        config.linkedin.clone(),
    ));
    let documents = Arc::new(DocumentService::new(Arc::new(PgDocumentRepository::new(
        pool.clone(),
    ))));
    let compilation = Arc::new(CompilationService::new(
        Arc::new(PgCompilationRepository::new(pool.clone())),
        documents.clone(),
    ));
    let cv_repository = Arc::new(PgCvRepository::new(pool.clone()));
    let cv_templates = Arc::new(CvTemplateService::new(cv_repository.clone()));
    let cv = Arc::new(CvService::new(cv_repository.clone(), cv_templates.clone()));
    let cv_render = Arc::new(CvRenderService::new(cv_templates.clone()));
    let cv_import = Arc::new(CvImportService::new(cv_repository));
    let state = AppState {
        pool,
        config,
        sessions,
        google,
        linkedin,
        documents,
        compilation,
        cv,
        cv_templates,
        cv_render,
        cv_import,
    };
    Router::new()
        .route("/health/live", get(live))
        .route("/health/ready", get(ready))
        .route(
            "/api/v1/sessions/anonymous",
            post(sessions::routes::anonymous_session),
        )
        .route("/api/v1/auth/register", post(sessions::routes::register))
        .route("/api/v1/auth/login", post(sessions::routes::login))
        .route(
            "/api/v1/auth/google/start",
            get(sessions::routes::google_start),
        )
        .route(
            "/api/v1/auth/google/callback",
            get(sessions::routes::google_callback),
        )
        .route(
            "/api/v1/auth/linkedin/start",
            get(sessions::routes::linkedin_start),
        )
        .route(
            "/api/v1/auth/linkedin/callback",
            get(sessions::routes::linkedin_callback),
        )
        .route("/api/v1/auth/logout", post(sessions::routes::logout))
        .route("/api/v1/auth/me", get(sessions::routes::me))
        .route("/api/v1/admin/users", get(sessions::routes::admin_users))
        .route("/api/v1/projects", post(documents::routes::create_project))
        .route(
            "/api/v1/projects/{project_id}/documents",
            post(documents::routes::create_document),
        )
        .route(
            "/api/v1/documents/{document_id}",
            get(documents::routes::get_document).put(documents::routes::update_document),
        )
        .route(
            "/api/v1/documents/{document_id}/compile",
            post(compilation::routes::create_compile_job),
        )
        .route(
            "/api/v1/compile-jobs/{job_id}",
            get(compilation::routes::get_compile_job)
                .delete(compilation::routes::cancel_compile_job),
        )
        .route(
            "/api/v1/artifacts/{artifact_id}",
            get(compilation::routes::get_artifact),
        )
        .route(
            "/api/v1/cv/session",
            get(cv::routes::get_session)
                .post(cv::routes::save_session)
                .put(cv::routes::save_session),
        )
        .route("/api/v1/cv/templates", get(cv::routes::list_templates))
        .route(
            "/api/v1/cv/templates/{template_id}/preview",
            get(cv::routes::template_preview),
        )
        .route("/api/v1/cv/render", post(cv::routes::render_cv))
        .route(
            "/api/v1/cv/import/linkedin/pending",
            get(cv::routes::get_pending_imports),
        )
        .route(
            "/api/v1/cv/import/linkedin/pending/{import_id}",
            delete(cv::routes::delete_pending_import),
        )
        .route(
            "/api/v1/cv/import/linkedin/pending/{import_id}/apply",
            post(cv::routes::apply_pending_import),
        )
        .with_state(state)
        .layer(DefaultBodyLimit::max(
            2 * cv::template::MAX_RENDER_DATA_BYTES,
        ))
        .layer(TimeoutLayer::with_status_code(
            axum::http::StatusCode::REQUEST_TIMEOUT,
            std::time::Duration::from_secs(10),
        ))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}

async fn live() -> axum::http::StatusCode {
    axum::http::StatusCode::NO_CONTENT
}

async fn ready(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<axum::http::StatusCode, error::AppError> {
    sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.pool)
        .await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
