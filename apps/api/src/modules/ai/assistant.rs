use axum::Router;
use axum::extract::{Path, State};
use axum::routing::get;
use http::Method;
use porte::Identity;
use serde::{Deserialize, Serialize};
use tronc::bulle::Request;
use tronc::json::Json;

use super::bulle::{connect, refused};
use crate::extract::id;
use crate::modules::boards::access::{self, Access};
use crate::state::AppState;

const DEFAULT_NAME: &str = "Bulle";

#[derive(Deserialize)]
struct Profile {
    name: Option<String>,
}

#[derive(Serialize)]
struct Assistant {
    name: String,
}

/// The name of `user_id`'s Bulle assistant, from Bulle's `GET /api/assistant` with a token
/// exchanged for them. "Bulle" when the name is empty or Bulle has no assistant for them.
pub async fn assistant(state: &AppState, user_id: i64) -> tronc::Result<String> {
    let (client, tokens) = connect(state, user_id)?;
    let profile: Profile = match client.call(&tokens, Request::new(Method::GET, ["assistant"])).await {
        Ok(profile) => profile,
        Err(error) if matches!(error.status(), Some(402 | 404)) => return Ok(DEFAULT_NAME.to_owned()),
        Err(error) => return Err(refused(error)),
    };
    let name = profile.name.as_deref().map(str::trim).unwrap_or_default();
    Ok(if name.is_empty() { DEFAULT_NAME } else { name }.to_owned())
}

async fn show(
    State(state): State<AppState>,
    who: Identity,
    Path(board): Path<String>,
) -> tronc::Result<Json<Assistant>> {
    let board = id(&board)?;
    access::check(&state, who.user_id, board, Access::Editor).await?;
    Ok(Json(Assistant { name: assistant(&state, who.user_id).await? }))
}

/// `GET /boards/{id}/ai/assistant`, behind `require_auth`: the caller's assistant name. It needs
/// editor access but no `ai.access`, so it never answers `402`.
pub fn routes() -> Router<AppState> {
    Router::new().route("/boards/{id}/ai/assistant", get(show))
}
