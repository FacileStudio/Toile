use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use porte::Identity;
use tronc::Result;
use tronc::json::Json;

use super::service;
use super::types::{Board, CreateRequest, ScopeQuery};
use crate::extract::{Params, id};
use crate::state::AppState;

async fn list(
    State(state): State<AppState>,
    who: Identity,
    Params(query): Params<ScopeQuery>,
) -> Result<Json<Vec<Board>>> {
    service::list(&state, who.user_id, query.space_id.as_deref()).await.map(Json)
}

async fn create(
    State(state): State<AppState>,
    who: Identity,
    Json(body): Json<CreateRequest>,
) -> Result<(StatusCode, Json<Board>)> {
    Ok((StatusCode::CREATED, Json(service::create(&state, who.user_id, body).await?)))
}

async fn read(State(state): State<AppState>, who: Identity, Path(raw): Path<String>) -> Result<Json<Board>> {
    service::read(&state, who.user_id, id(&raw)?).await.map(Json)
}

/// The board routes, mounted under `/api` behind `require_auth`.
pub fn routes() -> Router<AppState> {
    Router::new().route("/boards", get(list).post(create)).route("/boards/{id}", get(read))
}
