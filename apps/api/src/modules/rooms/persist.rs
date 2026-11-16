//! Storage: accepted updates append to `board_updates`; a snapshot replaces them in
//! `board_state` (salon decides when, from the policy in `host.rs`).

use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

async fn touch(tx: &mut PgConnection, id: Uuid) -> Result<bool, sqlx::Error> {
    let done = sqlx::query(
        "UPDATE boards SET updated_at = CASE WHEN deleted_at IS NULL THEN now() ELSE updated_at END WHERE id = $1",
    )
    .bind(id)
    .execute(&mut *tx)
    .await?;
    Ok(done.rows_affected() == 1)
}

/// Appends `updates` to the board's pending updates. `Ok(false)`: the board is gone.
pub async fn append(pool: &PgPool, id: Uuid, updates: &[Vec<u8>]) -> Result<bool, sqlx::Error> {
    let mut tx = pool.begin().await?;
    if !touch(&mut tx, id).await? {
        return Ok(false);
    }
    sqlx::query("INSERT INTO board_updates (board_id, body) SELECT $1, unnest($2::bytea[])")
        .bind(id)
        .bind(updates)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(true)
}

/// Stores `state` as the board's snapshot and drops its pending updates. `Ok(false)`: the board
/// is gone.
pub async fn store(pool: &PgPool, id: Uuid, state: &[u8]) -> Result<bool, sqlx::Error> {
    let mut tx = pool.begin().await?;
    if !touch(&mut tx, id).await? {
        return Ok(false);
    }
    sqlx::query(
        "INSERT INTO board_state (board_id, snapshot) VALUES ($1, $2) ON CONFLICT (board_id) DO UPDATE \
         SET snapshot = EXCLUDED.snapshot, version = board_state.version + 1, updated_at = now()",
    )
    .bind(id)
    .bind(state)
    .execute(&mut *tx)
    .await?;
    sqlx::query("DELETE FROM board_updates WHERE board_id = $1").bind(id).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(true)
}
