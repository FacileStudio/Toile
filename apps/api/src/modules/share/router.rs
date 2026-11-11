use std::sync::Arc;

use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, get};
use porte::Identity;
use serde::Deserialize;
use tronc::Result;
use tronc::http::{RateLimitConfig, RateLimiter, rate_limit};
use tronc::json::Json;

use super::guest;
use super::service::{self, Created, Link, Role};
use crate::extract::id;
use crate::state::AppState;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ShareRequest {
    role: Role,
}

async fn create(
    State(state): State<AppState>,
    who: Identity,
    Path(raw): Path<String>,
    Json(body): Json<ShareRequest>,
) -> Result<(StatusCode, Json<Created>)> {
    Ok((StatusCode::CREATED, Json(service::create(&state, who.user_id, id(&raw)?, body.role).await?)))
}

async fn list(State(state): State<AppState>, who: Identity, Path(raw): Path<String>) -> Result<Json<Vec<Link>>> {
    service::list(&state, who.user_id, id(&raw)?).await.map(Json)
}

async fn revoke(
    State(state): State<AppState>,
    who: Identity,
    Path((board, share)): Path<(String, String)>,
) -> Result<StatusCode> {
    service::revoke(&state, who.user_id, id(&board)?, id(&share)?).await.map(|()| StatusCode::NO_CONTENT)
}

/// The link routes, behind `require_auth`.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/boards/{id}/shares", get(list).post(create))
        .route("/boards/{id}/shares/{share}", delete(revoke))
}

/// The public reads of a link: 2 requests a second per client IP with a burst of 30.
pub fn public_routes() -> Router<AppState> {
    let limiter = Arc::new(RateLimiter::new(RateLimitConfig { limit: 2.0, burst: 30, ..RateLimitConfig::default() }));
    Router::new()
        .route("/share/{token}", get(guest::board))
        .route("/share/{token}/ws", get(guest::socket))
        .route_layer(axum::middleware::from_fn_with_state(limiter, rate_limit))
}
