use axum::Router;
use axum::extract::ws::WebSocketUpgrade;
use axum::extract::ws::rejection::WebSocketUpgradeRejection;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use http::request::Parts;
use porte::Identity;
use uuid::Uuid;

use super::command::Who;
use super::ws::{Caller, MAX_FRAME, serve};
use crate::extract::{db, id};
use crate::modules::boards::access::{self, Access};
use crate::state::AppState;

async fn display_name(state: &AppState, user_id: i64) -> tronc::Result<String> {
    let row: Option<(String, String)> = sqlx::query_as("SELECT name, email FROM porte_users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(db)?;
    Ok(row.map(|(name, email)| if name.trim().is_empty() { email } else { name }).unwrap_or_default())
}

/// Upgrades `upgrade` into a member socket of `board`; a request that is not a WebSocket
/// upgrade gets axum's rejection.
pub fn accept(
    state: AppState,
    board: Uuid,
    caller: Caller,
    upgrade: Result<WebSocketUpgrade, WebSocketUpgradeRejection>,
) -> Response {
    match upgrade {
        Ok(upgrade) => upgrade
            .max_message_size(MAX_FRAME + (1 << 20))
            .max_frame_size(MAX_FRAME + (1 << 20))
            .on_upgrade(move |socket| serve(socket, state, board, caller)),
        Err(rejection) => rejection.into_response(),
    }
}

async fn upgrade(
    State(state): State<AppState>,
    who: Identity,
    Path(raw): Path<String>,
    parts: Parts,
    upgrade: Result<WebSocketUpgrade, WebSocketUpgradeRejection>,
) -> tronc::Result<Response> {
    state.sessions.check_origin(&parts)?;
    let board = id(&raw)?;
    let have = access::check(&state, who.user_id, board, Access::Viewer).await?;
    let name = display_name(&state, who.user_id).await?;
    let member = Who { user_id: Some(who.user_id), share: None, name, writer: have >= Access::Editor };
    let caller = Caller { who: member, session_id: who.session_id };
    Ok(accept(state, board, caller, upgrade))
}

/// `GET /boards/{id}/ws`, behind `require_auth`: the y-websocket socket of one board.
pub fn routes() -> Router<AppState> {
    Router::new().route("/boards/{id}/ws", get(upgrade))
}
