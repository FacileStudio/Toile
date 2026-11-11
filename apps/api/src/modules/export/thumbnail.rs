use sqlx::PgPool;
use time::OffsetDateTime;
use tronc::Result;
use uuid::Uuid;

use super::raster;
use super::respond::{blocking, draw};
use super::source::decode;
use crate::extract::db;

/// The longest side of a thumbnail, in pixels.
pub const SIDE: u32 = 480;

type Row = (Option<Vec<u8>>, Option<OffsetDateTime>, Option<Vec<u8>>, Option<OffsetDateTime>);

/// `snapshot` (a stored board state, empty for none) drawn as a thumbnail PNG.
pub async fn render(snapshot: Vec<u8>) -> Result<Vec<u8>> {
    blocking(move || {
        let drawing = draw(&decode(&snapshot)?, None, Some("#ffffff"), "")?;
        raster::png(&drawing, 4.0, SIDE)
    })
    .await
}

/// Renders `snapshot` and stores it as board `id`'s thumbnail.
pub async fn refresh(pool: &PgPool, id: Uuid, snapshot: Vec<u8>) -> Result<Vec<u8>> {
    let png = render(snapshot).await?;
    sqlx::query("UPDATE boards SET thumbnail = $2, thumbnail_at = now() WHERE id = $1")
        .bind(id)
        .bind(&png)
        .execute(pool)
        .await
        .map_err(db)?;
    Ok(png)
}

/// Refreshes board `id`'s thumbnail in the background after a snapshot; a failure is logged.
pub fn spawn_refresh(pool: PgPool, id: Uuid, snapshot: Vec<u8>) {
    tokio::spawn(async move {
        if let Err(error) = refresh(&pool, id, snapshot).await {
            tracing::warn!(board = %id, error = %error.message, "export: thumbnail refresh failed");
        }
    });
}

/// Board `id`'s thumbnail: the stored one while it is newer than the stored snapshot, else
/// rendered from that snapshot and stored. The caller has checked access.
pub async fn current(pool: &PgPool, id: Uuid) -> Result<Vec<u8>> {
    let row: Row = sqlx::query_as(
        "SELECT b.thumbnail, b.thumbnail_at, s.snapshot, s.updated_at FROM boards b \
         LEFT JOIN board_state s ON s.board_id = b.id WHERE b.id = $1",
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(db)?;
    match row {
        (Some(png), Some(at), _, stored) if stored.is_none_or(|stored| at >= stored) => Ok(png),
        (_, _, snapshot, _) => refresh(pool, id, snapshot.unwrap_or_default()).await,
    }
}
