use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Response;
use porte::Identity;
use tronc::json::Json;
use tronc::{Error, Result};

use super::access::{self, Access};
use super::types::{Board, CreateRequest, DeleteQuery, NameRequest, ScopeQuery};
use super::{service, trash};
use crate::extract::{Params, id};
use crate::state::AppState;

pub async fn list(
    State(state): State<AppState>,
    who: Identity,
    Params(q): Params<ScopeQuery>,
) -> Result<Json<Vec<Board>>> {
    service::list(&state, who.user_id, q.space_id.as_deref(), q.trash).await.map(Json)
}

pub async fn create(
    State(state): State<AppState>,
    who: Identity,
    Json(body): Json<CreateRequest>,
) -> Result<(StatusCode, Json<Board>)> {
    Ok((StatusCode::CREATED, Json(service::create(&state, who.user_id, body).await?)))
}

pub async fn read(State(state): State<AppState>, who: Identity, Path(raw): Path<String>) -> Result<Json<Board>> {
    service::read(&state, who.user_id, id(&raw)?).await.map(Json)
}

pub async fn rename(
    State(state): State<AppState>,
    who: Identity,
    Path(raw): Path<String>,
    Json(body): Json<NameRequest>,
) -> Result<Json<Board>> {
    service::rename(&state, who.user_id, id(&raw)?, body).await.map(Json)
}

pub async fn remove(
    State(state): State<AppState>,
    who: Identity,
    Path(raw): Path<String>,
    Params(q): Params<DeleteQuery>,
) -> Result<StatusCode> {
    let board = id(&raw)?;
    match q.permanent {
        true => trash::purge(&state, who.user_id, board).await?,
        false => trash::trash(&state, who.user_id, board).await?,
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn duplicate(
    State(state): State<AppState>,
    who: Identity,
    Path(raw): Path<String>,
    body: axum::body::Bytes,
) -> Result<(StatusCode, Json<Board>)> {
    let request = match body.is_empty() {
        true => NameRequest::default(),
        false => serde_json::from_slice(&body).map_err(|error| Error::invalid(error.to_string()))?,
    };
    Ok((StatusCode::CREATED, Json(service::duplicate(&state, who.user_id, id(&raw)?, request).await?)))
}

pub async fn restore(State(state): State<AppState>, who: Identity, Path(raw): Path<String>) -> Result<Json<Board>> {
    trash::restore(&state, who.user_id, id(&raw)?).await.map(Json)
}

pub async fn thumbnail(State(state): State<AppState>, who: Identity, Path(raw): Path<String>) -> Result<Response> {
    let board = id(&raw)?;
    access::check(&state, who.user_id, board, Access::Viewer).await?;
    let png = crate::modules::export::thumbnail::current(&state.pool, board).await?;
    Response::builder()
        .header(http::header::CONTENT_TYPE, "image/png")
        .header(http::header::CACHE_CONTROL, "private, max-age=86400")
        .body(Body::from(png))
        .map_err(|error| Error::internal("thumbnail response", error))
}
