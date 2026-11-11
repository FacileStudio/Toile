//! Storage: accepted updates append to `board_updates`; a snapshot replaces them in
//! `board_state` every [`SNAPSHOT_EVERY`] updates or [`SNAPSHOT_AFTER`].

use sqlx::{PgConnection, PgPool};
use tokio::time::Instant;
use uuid::Uuid;

use super::room::{FLUSH, Room, SNAPSHOT_AFTER, SNAPSHOT_BYTES, SNAPSHOT_EVERY};

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

impl Room {
    pub(super) fn snapshot_due(&self, now: Instant) -> bool {
        let pending = self.core.log.len();
        pending >= SNAPSHOT_EVERY
            || self.core.bytes >= SNAPSHOT_BYTES
            || self.since.is_some_and(|at| at + SNAPSHOT_AFTER <= now)
    }

    fn stored(&mut self, outcome: Result<bool, sqlx::Error>, what: &str) -> bool {
        match outcome {
            Ok(true) => true,
            Ok(false) => {
                self.vanish();
                false
            }
            Err(error) => {
                tracing::error!(board = %self.id, error = %error, "rooms: {what} failed, retrying");
                self.flush_at = Some(Instant::now() + FLUSH * 4);
                false
            }
        }
    }

    /// Appends the updates accepted since the last flush.
    pub(super) async fn flush(&mut self) {
        self.flush_at = None;
        if self.flushed >= self.core.log.len() {
            return;
        }
        let outcome = append(&self.state.pool, self.id, &self.core.log[self.flushed..]).await;
        if self.stored(outcome, "append") {
            self.flushed = self.core.log.len();
        }
    }

    /// Writes a snapshot of the whole board and drops the pending updates it covers.
    pub(super) async fn snapshot(&mut self) {
        if self.since.is_none() && self.core.log.is_empty() {
            return;
        }
        let state = self.core.state();
        let outcome = store(&self.state.pool, self.id, &state).await;
        if self.stored(outcome, "snapshot") {
            self.core.compacted(state);
            (self.flushed, self.flush_at, self.since) = (0, None, None);
        }
    }
}
