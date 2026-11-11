use sqlx::PgPool;
use uuid::Uuid;

/// A stored board: its snapshot (empty when none was written) and its pending updates, oldest
/// first.
pub type Stored = (Vec<u8>, Vec<Vec<u8>>);

/// Reads a live board's snapshot and pending updates; `None` when the board is missing or in
/// the trash.
pub async fn load(pool: &PgPool, id: Uuid) -> Result<Option<Stored>, sqlx::Error> {
    let row: Option<(Option<Vec<u8>>,)> = sqlx::query_as(
        "SELECT s.snapshot FROM boards b LEFT JOIN board_state s ON s.board_id = b.id \
         WHERE b.id = $1 AND b.deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    let Some((snapshot,)) = row else { return Ok(None) };
    let updates: Vec<Vec<u8>> = sqlx::query_scalar("SELECT body FROM board_updates WHERE board_id = $1 ORDER BY id")
        .bind(id)
        .fetch_all(pool)
        .await?;
    Ok(Some((snapshot.unwrap_or_default(), updates)))
}
