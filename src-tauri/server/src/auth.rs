use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{StatusCode, Request},
    middleware::Next,
    response::{Response, IntoResponse},
};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use hex;
use tracing::warn;
use std::sync::Arc;

use crate::db::repo::App;
use crate::AppState;

use governor::{Quota, RateLimiter, state::keyed::DashMapStateStore, clock::DefaultClock};
use std::num::NonZeroU32;

lazy_static::lazy_static! {
    static ref RATE_LIMITER: RateLimiter<String, DashMapStateStore<String>, DefaultClock> = 
        RateLimiter::keyed(Quota::per_minute(NonZeroU32::new(100).unwrap()));
}

pub fn hash_api_key(secret: &str, key: &str) -> String {
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(key.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

pub async fn auth_middleware(
    state: axum::extract::State<Arc<AppState>>,
    req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // We need to handle the Option<String> from query or Option<&str> from header.
    // Let's simplify.
    let api_key_header = req.headers().get("X-API-KEY")
        .or_else(|| req.headers().get("GUEST_API_KEY"))
        .or_else(|| req.headers().get("guest_api_key")) // Just in case
        .and_then(|v| v.to_str().ok()).map(|s| s.to_string());
    
    let api_key_query = req.uri().query().and_then(|query| {
        url::form_urlencoded::parse(query.as_bytes())
            .find(|(k, _)| k == "api_key" || k == "guest_api_key")
            .map(|(_, v)| v.into_owned())
    });

    let api_key = api_key_header.or(api_key_query);

    let api_key = match api_key {
        Some(key) => key,
        None => {
            warn!("Missing API key");
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    let server_secret = &state.config.security.server_secret;
    let key_hash = hash_api_key(server_secret, &api_key);

    let app = state.repo.get_app_by_key_hash(&key_hash).await.map_err(|e| {
        warn!("Database error during auth: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if let Some(app) = app {
        if !app.is_active {
             warn!("Inactive API key used: {}", app.id);
             return Err(StatusCode::UNAUTHORIZED);
        }
        
        // --- Rate Limiting ---
        if RATE_LIMITER.check_key(&app.id).is_err() {
            return Err(StatusCode::TOO_MANY_REQUESTS);
        }
        // ---------------------
        
        let _ = state.repo.update_app_last_used(&app.id).await;

        let mut req = req;
        req.extensions_mut().insert(app);
        Ok(next.run(req).await)
    } else {
        warn!("Invalid API key");
        Err(StatusCode::UNAUTHORIZED)
    }
}

// Extractor to easily get the App in handlers
pub struct AuthenticatedApp(pub App);

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedApp
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut axum::http::request::Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let app = parts
            .extensions
            .get::<App>()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?; // Should be there due to middleware
        Ok(AuthenticatedApp(App {
            id: app.id.clone(),
            api_key_hash: app.api_key_hash.clone(),
            name: app.name.clone(),
            is_active: app.is_active,
            created_at: app.created_at,
            last_used_at: app.last_used_at,
        }))
    }
}
