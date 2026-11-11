use std::sync::Arc;

use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, patch, post};
use porte::Identity;
use tronc::http::{RateLimitConfig, RateLimiter, rate_limit};
use tronc::json::Json;
use tronc::{Error, Result};

use super::service::{self, Author};
use super::types::{Comment, CreateRequest, GuestRequest, PatchRequest};
use crate::extract::id;
use crate::modules::boards::access::{self, Access};
use crate::modules::share::guest::guest_name;
use crate::modules::share::service::{Role, resolve};
use crate::state::AppState;

async fn list(State(state): State<AppState>, who: Identity, Path(raw): Path<String>) -> Result<Json<Vec<Comment>>> {
    let board = id(&raw)?;
    access::check(&state, who.user_id, board, Access::Viewer).await?;
    service::list(&state, board, Some(who.user_id)).await.map(Json)
}

async fn create(
    State(state): State<AppState>,
    who: Identity,
    Path(raw): Path<String>,
    Json(body): Json<CreateRequest>,
) -> Result<(StatusCode, Json<Comment>)> {
    let board = id(&raw)?;
    access::check(&state, who.user_id, board, Access::Commenter).await?;
    Ok((StatusCode::CREATED, Json(service::create(&state, board, Author::User(who.user_id), body).await?)))
}

async fn edit(
    State(state): State<AppState>,
    who: Identity,
    Path((board, comment)): Path<(String, String)>,
    Json(body): Json<PatchRequest>,
) -> Result<Json<Comment>> {
    service::update(&state, who.user_id, id(&board)?, id(&comment)?, body).await.map(Json)
}

async fn remove(
    State(state): State<AppState>,
    who: Identity,
    Path((b, c)): Path<(String, String)>,
) -> Result<StatusCode> {
    service::delete(&state, who.user_id, id(&b)?, id(&c)?).await.map(|()| StatusCode::NO_CONTENT)
}

async fn guest_list(State(state): State<AppState>, Path(token): Path<String>) -> Result<Json<Vec<Comment>>> {
    let share = resolve(&state, &token).await?;
    service::list(&state, share.board_id, None).await.map(Json)
}

async fn guest_create(
    State(state): State<AppState>,
    Path(token): Path<String>,
    Json(body): Json<GuestRequest>,
) -> Result<(StatusCode, Json<Comment>)> {
    let name = guest_name(&body.name)?;
    let share = resolve(&state, &token).await?;
    if share.role == Role::View {
        return Err(Error::forbidden("this link may not comment"));
    }
    let request = CreateRequest { anchor: body.anchor, body: body.body, parent: body.parent };
    let author = Author::Guest(name, share.id);
    Ok((StatusCode::CREATED, Json(service::create(&state, share.board_id, author, request).await?)))
}

/// The comment routes, behind `require_auth`.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/boards/{id}/comments", get(list).post(create))
        .route("/boards/{id}/comments/{comment}", patch(edit).delete(remove))
}

/// A guest's comment routes: reads share the link limit; posting is held to one comment every
/// two seconds per client IP, with a burst of 10.
pub fn public_routes() -> Router<AppState> {
    let reads = Arc::new(RateLimiter::new(RateLimitConfig { limit: 2.0, burst: 30, ..RateLimitConfig::default() }));
    let posts = Arc::new(RateLimiter::new(RateLimitConfig { limit: 0.5, burst: 10, ..RateLimitConfig::default() }));
    let read = get(guest_list).layer(axum::middleware::from_fn_with_state(reads, rate_limit));
    let write = post(guest_create).layer(axum::middleware::from_fn_with_state(posts, rate_limit));
    Router::new().route("/share/{token}/comments", read.merge(write))
}
