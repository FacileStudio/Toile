//! Moving a board between scopes: the caller's personal boards and the spaces they edit in.

use tronc::Result;
use uuid::Uuid;

use super::access::{self, Access};
use super::service::fetch;
use super::types::{Board, BoardRow, columns};
use crate::extract::db;
use crate::state::AppState;

/// Moves a board to `space_id`, or to the caller's personal boards when `None`. Needs `owner` on
/// the board and `editor` in the destination. A board moved to personal becomes the caller's. The
/// board's room closes so every client reconnects under its new access. Moving a board to the
/// scope it is already in changes nothing.
pub async fn relocate(state: &AppState, user_id: i64, id: Uuid, space_id: Option<&str>) -> Result<Board> {
    access::check(state, user_id, id, Access::Owner).await?;
    let have = access::scope(state, user_id, space_id).await?;
    access::require(have, Access::Editor)?;
    let current = fetch(state, id).await?;
    if current.space_id.as_deref() == space_id {
        return Ok(Board::from_row(current, have));
    }
    let row = sqlx::query_as::<_, BoardRow>(concat!(
        "UPDATE boards SET space_id = $2, updated_at = now(), ",
        "owner_user_id = CASE WHEN $2::text IS NULL THEN $3 ELSE owner_user_id END ",
        "WHERE id = $1 AND deleted_at IS NULL RETURNING ",
        columns!()
    ))
    .bind(id)
    .bind(space_id)
    .bind(user_id)
    .fetch_one(&state.pool)
    .await
    .map_err(db)?;
    state.hub.close(&[id]).await;
    Ok(Board::from_row(row, have))
}
