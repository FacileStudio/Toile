use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::post;
use porte::Identity;
use serde::{Deserialize, Serialize};
use tronc::json::Json;
use tronc::{Error, Result};

use super::opus::{self, Task};
use super::sticky;
use crate::extract::id;
use crate::modules::boards::access::{self, Access};
use crate::state::AppState;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    project: Option<serde_json::Value>,
}

#[derive(Serialize)]
struct Answer {
    task: Task,
    card: String,
}

fn project(raw: Option<serde_json::Value>) -> Result<i64> {
    let value = raw.ok_or_else(|| Error::invalid("project is required: the Opus project id"))?;
    value
        .as_i64()
        .or_else(|| value.as_str()?.trim().parse().ok())
        .filter(|id| *id > 0)
        .ok_or_else(|| Error::invalid("project must be an Opus project id"))
}

async fn create(
    State(state): State<AppState>,
    who: Identity,
    Path((board, shape)): Path<(String, String)>,
    Json(body): Json<Request>,
) -> Result<(StatusCode, Json<Answer>)> {
    let board = id(&board)?;
    access::check(&state, who.user_id, board, Access::Editor).await?;
    let project = project(body.project)?;
    let found = sticky::read(&state, board, &shape).await?;
    let task = opus::create(&state, who.user_id, project, &found.text).await?;
    let card = sticky::new_id();
    let edit = sticky::card(card.clone(), &found, task.url.clone(), task.title.clone());
    state.hub.write(&state, board, edit).await?;
    Ok((StatusCode::CREATED, Json(Answer { task, card })))
}

/// `POST /boards/{id}/shapes/{shape}/task`, behind `require_auth`.
pub fn routes() -> Router<AppState> {
    Router::new().route("/boards/{id}/shapes/{shape}/task", post(create))
}
