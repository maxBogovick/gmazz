use axum::{
    routing::{get, post, delete},
    Router,
    middleware,
    response::IntoResponse,
};
use tower_http::trace::TraceLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::cors::{CorsLayer, Any};
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
    ),
    components(
        schemas(FileResponse, ListParams, ArchiveResponse, ArchiveStatusResponse, CreateKeyRequest, CreateKeyResponse, AppKeyResponse, ReleaseResponse, CreateReleaseRequest)
    ),
    tags(
        (name = "files", description = "File management endpoints"),
        (name = "archives", description = "Archive endpoints"),
        (name = "admin", description = "Administrative endpoints"),
        (name = "releases", description = "Release management endpoints")
    ),
    security(
        ("api_key" = []),
        ("admin_secret" = [])
    )
)]
struct ApiDoc;

pub async fn app(state: Arc<AppState>) -> Router {
    // API Routes (Protected by API Key)
    let api_routes = Router::new()
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
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

    // Admin Routes (Protected by Admin Secret)
    let admin_routes = Router::new()
        .route("/keys", post(create_key_handler))
        .route("/keys", get(list_keys_handler))
        .route("/keys/:id/revoke", axum::routing::put(revoke_key_handler))
        .route("/keys/:id/activate", axum::routing::put(activate_key_handler))
        .layer(middleware::from_fn_with_state(state.clone(), admin_auth_middleware));

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
            HeaderName::from_static("x-api-key"),
            HeaderName::from_static("x-admin-secret"),
            HeaderName::from_static("x-file-name"),
        ]);

    let mut router = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/health", get(health_check))
        .nest("/v1", api_routes)
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
