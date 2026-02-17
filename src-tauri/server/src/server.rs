use axum::{
    routing::{get, post, delete},
    Router,
    middleware,
    response::IntoResponse,
};
use tower_http::trace::TraceLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::cors::{CorsLayer, Any, AllowOrigin};
use std::sync::Arc;
use crate::{AppState};
use crate::api::files::{upload_handler, download_handler, list_handler, delete_handler, get_file_meta_handler, FileResponse, ListParams};
use crate::api::archives::{create_archive_handler, get_archive_handler, ArchiveResponse, ArchiveStatusResponse};
use crate::api::admin::{create_key_handler, list_keys_handler, revoke_key_handler, activate_key_handler, CreateKeyRequest, CreateKeyResponse, AppKeyResponse};
use crate::api::releases::{create_release_handler, get_latest_release_handler, list_releases_handler, ReleaseResponse, CreateReleaseRequest};
use crate::auth::auth_middleware;
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use std::future::ready;
use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::http::{StatusCode, HeaderName};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use governor::{Quota, RateLimiter, state::keyed::DashMapStateStore, clock::DefaultClock};
use std::num::NonZeroU32;

use crate::api::public::{list_public_notes_handler, get_public_note_handler, PublicNote, NoteFilter};
use crate::api::settings::{get_all_settings_handler, get_setting_handler, set_setting_handler, delete_setting_handler, SettingResponse, SetSettingRequest, AllSettingsResponse};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::api::files::upload_handler,
        crate::api::files::download_handler,
        crate::api::files::list_handler,
        crate::api::files::delete_handler,
        crate::api::files::get_file_meta_handler,
        crate::api::archives::create_archive_handler,
        crate::api::archives::get_archive_handler,
        crate::api::admin::create_key_handler,
        crate::api::admin::list_keys_handler,
        crate::api::admin::revoke_key_handler,
        crate::api::admin::activate_key_handler,
        crate::api::releases::create_release_handler,
        crate::api::releases::get_latest_release_handler,
        crate::api::releases::list_releases_handler,
        crate::api::releases_actions::activate_release_handler,
        crate::api::public::list_public_notes_handler,
        crate::api::public::get_public_note_handler,
        crate::api::settings::get_all_settings_handler,
        crate::api::settings::get_setting_handler,
        crate::api::settings::set_setting_handler,
        crate::api::settings::delete_setting_handler,
    ),
    components(
        schemas(FileResponse, ListParams, ArchiveResponse, ArchiveStatusResponse, CreateKeyRequest, CreateKeyResponse, AppKeyResponse, ReleaseResponse, CreateReleaseRequest, PublicNote, NoteFilter, SettingResponse, SetSettingRequest, AllSettingsResponse)
    ),
    tags(
        (name = "files", description = "File management endpoints"),
        (name = "archives", description = "Archive endpoints"),
        (name = "admin", description = "Administrative endpoints"),
        (name = "releases", description = "Release management endpoints"),
        (name = "public", description = "Public access endpoints")
    ),
    security(
        ("api_key" = []),
        ("admin_secret" = [])
    )
)]
struct ApiDoc;

pub async fn app(state: Arc<AppState>) -> Router {
    // Note: Public routes should technically bypass `auth_middleware` if they are truly public without key.
    // The current setup protects ALL `/v1/*` with `auth_middleware`.
    // We need to separate them.
    
    let public_routes = Router::new()
        .route("/notes", get(list_public_notes_handler))
        .route("/notes/:id", get(get_public_note_handler))
        .route("/auth/guest", post(crate::api::auth::create_guest_key_handler))
        .route("/settings", get(get_all_settings_handler))
        .route("/settings/:key", get(get_setting_handler))
        .layer(middleware::from_fn(public_rate_limit));

    let protected_api_routes = Router::new()
        .route("/files", post(upload_handler))
        .route("/files", get(list_handler))
        .route("/files/:id", get(download_handler))
        .route("/files/:id", delete(delete_handler))
        .route("/files/:id/meta", get(get_file_meta_handler))
        .route("/archives", post(create_archive_handler))
        .route("/archives/:id", get(get_archive_handler))
        .route("/releases", post(create_release_handler))
        .route("/releases", get(list_releases_handler))
        .route("/releases/latest", get(get_latest_release_handler))
        .route("/releases/:id/activate", axum::routing::put(crate::api::releases_actions::activate_release_handler))
        .route("/settings/:key", axum::routing::put(set_setting_handler))
        .route("/settings/:key", delete(delete_setting_handler))
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

    // Admin Routes (Protected by Admin Secret)
    let admin_routes = Router::new()
        .route("/keys", post(create_key_handler))
        .route("/keys", get(list_keys_handler))
        .route("/keys/:id/revoke", axum::routing::put(revoke_key_handler))
        .route("/keys/:id/activate", axum::routing::put(activate_key_handler))
        .layer(middleware::from_fn_with_state(state.clone(), admin_auth_middleware));

    let cors = if state.config.server.allowed_origins.is_empty() {
        CorsLayer::new().allow_origin(Any)
    } else {
        let origins: Vec<axum::http::HeaderValue> = state
            .config
            .server
            .allowed_origins
            .iter()
            .filter_map(|origin| origin.parse().ok())
            .collect();
        CorsLayer::new().allow_origin(AllowOrigin::list(origins))
    }
        .allow_methods(Any)
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
            HeaderName::from_static("x-api-key"),
            HeaderName::from_static("guest-api-key"), // Correct header name casing for from_static? standard is lowercase for HTTP/2 but from_static expects lowercase.
            // Wait, the prompt said "GUEST_API_KEY". HTTP headers are case-insensitive but convention is dashes.
            // If the user sends "GUEST_API_KEY" literally (with underscores), I should allow that.
            HeaderName::from_static("guest_api_key"), 
            HeaderName::from_static("x-admin-secret"),
            HeaderName::from_static("x-file-name"),
        ]);

    let mut router = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/health", get(health_check))
        .nest("/v1/public", public_routes)
        .nest("/v1", protected_api_routes)
        .nest("/admin", admin_routes)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .layer(RequestBodyLimitLayer::new(state.config.server.request_body_limit_bytes))
        .layer(middleware::from_fn(track_metrics))
        .with_state(state);

    // Optional metrics endpoint if recorder can be installed
    if let Ok(handle) = PrometheusBuilder::new().install_recorder() {
        router = router.route("/metrics", get(move || ready(handle.render())));
    }

    router
}

lazy_static::lazy_static! {
    static ref PUBLIC_RATE_LIMITER: RateLimiter<String, DashMapStateStore<String>, DefaultClock> =
        RateLimiter::keyed(Quota::per_minute(NonZeroU32::new(120).unwrap()));
}

async fn public_rate_limit(req: Request, next: Next) -> Result<impl IntoResponse, StatusCode> {
    let key = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "public".to_string());

    if PUBLIC_RATE_LIMITER.check_key(&key).is_err() {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    Ok(next.run(req).await)
}

async fn track_metrics(req: Request, next: Next) -> impl axum::response::IntoResponse {
    let start = std::time::Instant::now();
    let path = req.uri().path().to_owned();
    let method = req.method().clone();

    let response = next.run(req).await;

    // let latency = start.elapsed().as_secs_f64();
    // let status = response.status().as_u16().to_string();

    // metrics::increment_counter!("http_requests_total", "method" => method.to_string(), "path" => path.clone(), "status" => status.clone());
    // metrics::histogram!("http_request_duration_seconds", latency, "method" => method.to_string(), "path" => path, "status" => status);

    response
}

async fn health_check() -> &'static str {
    "OK"
}

async fn admin_auth_middleware(
    State(state): State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, StatusCode> {
    let secret = req.headers().get("X-ADMIN-SECRET")
        .and_then(|v| v.to_str().ok());

    match secret {
        Some(s) if s == state.config.security.admin_secret => Ok(next.run(req).await),
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}
