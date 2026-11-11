//! The trash: a trashed board is invisible to every route but these, and is purged after
//! [`RETENTION`].

use std::time::Duration;

use tokio_util::sync::CancellationToken;
use tronc::{Error, Result};
use uuid::Uuid;

use super::access::{self, Access};
use super::types::Board;
use crate::extract::db;
use crate::state::AppState;

/// How long a trashed board waits before it is deleted for good.
pub const RETENTION: &str = "30 days";
/// How often the purge runs.
pub const PURGE_EVERY: Duration = Duration::from_secs(3600);

/// Moves a board to the trash and closes its room; needs `owner`.
pub async fn trash(state: &AppState, user_id: i64, id: Uuid) -> Result<()> {
    access::check(state, user_id, id, Access::Owner).await?;
    sqlx::query("UPDATE boards SET deleted_at = now() WHERE id = $1 AND deleted_at IS NULL")
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(db)?;
    state.hub.close(&[id]).await;
    Ok(())
}

async fn trashed(state: &AppState, user_id: i64, id: Uuid) -> Result<Access> {
    let have = access::lookup(state, user_id, id, true).await?;
    access::require(have, Access::Owner).map_err(|_| Error::not_found("board not found"))?;
    Ok(have)
}

/// Takes a board out of the trash; needs `owner`. A board whose space is gone stays trashed.
pub async fn restore(state: &AppState, user_id: i64, id: Uuid) -> Result<Board> {
    let have = trashed(state, user_id, id).await?;
    let row = sqlx::query_as::<_, super::types::BoardRow>(concat!(
        "UPDATE boards SET deleted_at = NULL, updated_at = now() WHERE id = $1 RETURNING ",
        super::types::columns!()
    ))
    .bind(id)
    .fetch_one(&state.pool)
    .await
    .map_err(db)?;
    Ok(Board::from_row(row, have))
}

/// Deletes a trashed board for good; needs `owner`. A live board must be trashed first.
pub async fn purge(state: &AppState, user_id: i64, id: Uuid) -> Result<()> {
    trashed(state, user_id, id).await?;
    sqlx::query("DELETE FROM boards WHERE id = $1 AND deleted_at IS NOT NULL")
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(db)?;
    Ok(())
}

/// Deletes every board trashed more than [`RETENTION`] ago; answers how many.
pub async fn sweep(state: &AppState) -> std::result::Result<u64, sqlx::Error> {
    let done = sqlx::query("DELETE FROM boards WHERE deleted_at < now() - $1::interval")
        .bind(RETENTION)
        .execute(&state.pool)
        .await?;
    Ok(done.rows_affected())
}

/// Runs [`sweep`] every [`PURGE_EVERY`] until `token` is cancelled.
pub async fn run(state: AppState, token: CancellationToken) {
    let mut every = tokio::time::interval(PURGE_EVERY);
    loop {
        tokio::select! {
            () = token.cancelled() => return,
            _ = every.tick() => match sweep(&state).await {
                Ok(0) => {}
                Ok(purged) => tracing::info!(purged, "boards: purged the trash"),
                Err(error) => tracing::error!(error = %error, "boards: purge failed"),
            },
        }
    }
}
