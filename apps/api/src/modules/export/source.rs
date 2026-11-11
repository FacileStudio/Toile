use tronc::{Error, Result};
use uuid::Uuid;
use yrs::Doc;

use crate::modules::board;
use crate::modules::boards::service;
use crate::modules::boards::types::Board;
use crate::state::AppState;

/// A board document from stored or live Yjs bytes.
pub fn decode(state: &[u8]) -> Result<Doc> {
    board::from_state(state, []).map_err(|e| Error::internal("board state unreadable", e.to_string()))
}

/// Board `id` as the caller may see it, with its live document: `404` without access.
pub async fn board(state: &AppState, user_id: i64, id: Uuid) -> Result<(Board, Doc)> {
    let board = service::read(state, user_id, id).await?;
    let live = state.hub.read(state, id).await?;
    Ok((board, decode(&live)?))
}
