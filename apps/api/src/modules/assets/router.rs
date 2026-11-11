use std::sync::Arc;

use axum::Router;
use axum::extract::{DefaultBodyLimit, Multipart, Path, State};
use axum::response::Response;
use axum::routing::{get, post};
use http::HeaderMap;
use porte::Identity;
use tronc::http::{RateLimitConfig, RateLimiter, rate_limit};
use tronc::{Error, Result};

use super::{serve, upload};
use crate::extract::id;
use crate::modules::boards::access::{self, Access};
use crate::modules::share::service as share;
use crate::state::AppState;

async fn create(
    State(state): State<AppState>,
    who: Identity,
    Path(raw): Path<String>,
    form: Multipart,
) -> Result<Response> {
    upload::upload(&state, who.user_id, id(&raw)?, form).await
}

async fn read(
    State(state): State<AppState>,
    who: Identity,
    Path((board, asset)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Response> {
    let board = id(&board)?;
    access::check(&state, who.user_id, board, Access::Viewer).await?;
    serve::serve(&state, board, id(&asset)?, &headers).await
}

async fn guest_read(
    State(state): State<AppState>,
    Path((token, asset)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Response> {
    let link = share::resolve(&state, &token).await?;
    serve::serve(&state, link.board_id, id(&asset)?, &headers).await
}

async fn guest_upload() -> Error {
    Error::forbidden("guests cannot upload images")
}

/// The upload and read routes, behind `require_auth`.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/boards/{id}/assets", post(create).layer(DefaultBodyLimit::max(upload::MAX_BYTES + (1 << 20))))
        .route("/boards/{id}/assets/{asset}", get(read))
}

/// A link holder's image reads, 20 a second per client IP with a burst of 200; a guest upload
/// is always 403.
pub fn public_routes() -> Router<AppState> {
    let limiter = Arc::new(RateLimiter::new(RateLimitConfig { limit: 20.0, burst: 200, ..RateLimitConfig::default() }));
    Router::new()
        .route(
            "/share/{token}/assets/{asset}",
            get(guest_read).route_layer(axum::middleware::from_fn_with_state(limiter, rate_limit)),
        )
        .route("/share/{token}/assets", post(guest_upload))
}
