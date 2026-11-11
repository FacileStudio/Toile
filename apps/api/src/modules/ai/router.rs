use axum::Router;
use axum::extract::{Path, State};
use axum::routing::post;
use porte::Identity;
use serde::{Deserialize, Serialize};
use tronc::json::Json;

use super::bulle::{Refusal, gate};
use super::cluster::{self, Framed};
use super::generate::{self, Draw};
use super::summarize;
use crate::extract::id;
use crate::scene::input::Placed;
use crate::state::AppState;

#[derive(Serialize)]
struct Summary {
    markdown: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ClusterIn {
    shapes: Vec<String>,
}

#[derive(Serialize)]
struct ClusterOut {
    frames: Vec<Framed>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GenerateIn {
    prompt: String,
    kind: Draw,
}

#[derive(Serialize)]
struct GenerateOut {
    created: Vec<Placed>,
}

async fn summarize(
    State(state): State<AppState>,
    who: Identity,
    Path(board): Path<String>,
) -> Result<Json<Summary>, Refusal> {
    let board = id(&board)?;
    gate(&state, who.user_id, board).await?;
    Ok(Json(Summary { markdown: summarize::summarize(&state, who.user_id, board).await? }))
}

async fn cluster(
    State(state): State<AppState>,
    who: Identity,
    Path(board): Path<String>,
    Json(body): Json<ClusterIn>,
) -> Result<Json<ClusterOut>, Refusal> {
    let board = id(&board)?;
    gate(&state, who.user_id, board).await?;
    Ok(Json(ClusterOut { frames: cluster::cluster(&state, who.user_id, board, &body.shapes).await? }))
}

async fn generate(
    State(state): State<AppState>,
    who: Identity,
    Path(board): Path<String>,
    Json(body): Json<GenerateIn>,
) -> Result<Json<GenerateOut>, Refusal> {
    let board = id(&board)?;
    gate(&state, who.user_id, board).await?;
    let created = generate::generate(&state, who.user_id, board, &body.prompt, body.kind).await?;
    Ok(Json(GenerateOut { created }))
}

/// `POST /boards/{id}/ai/summarize`, `/ai/cluster` and `/ai/generate`, behind `require_auth`.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/boards/{id}/ai/summarize", post(summarize))
        .route("/boards/{id}/ai/cluster", post(cluster))
        .route("/boards/{id}/ai/generate", post(generate))
}
