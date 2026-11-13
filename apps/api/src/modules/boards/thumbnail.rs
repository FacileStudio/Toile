use axum::body::{Body, Bytes};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Response;
use porte::Identity;
use tronc::{Error, Result};

use super::access::{self, Access};
use crate::extract::{db, id};
use crate::state::AppState;

pub async fn thumbnail(State(state): State<AppState>, who: Identity, Path(raw): Path<String>) -> Result<Response> {
    let board = id(&raw)?;
    access::check(&state, who.user_id, board, Access::Viewer).await?;
    let png: Option<Vec<u8>> = sqlx::query_scalar("SELECT thumbnail FROM boards WHERE id = $1")
        .bind(board)
        .fetch_one(&state.pool)
        .await
        .map_err(db)?;
    let png = png.ok_or_else(|| Error::not_found("no thumbnail yet"))?;
    Response::builder()
        .header(http::header::CONTENT_TYPE, "image/png")
        .header(http::header::CACHE_CONTROL, "private, max-age=86400")
        .body(Body::from(png))
        .map_err(|error| Error::internal("thumbnail response", error))
}

const PNG_MAGIC: &[u8] = b"\x89PNG\r\n\x1a\n";

const THUMBNAIL_MAX: usize = 512 << 10;

pub async fn store_thumbnail(
    State(state): State<AppState>,
    who: Identity,
    Path(raw): Path<String>,
    body: Bytes,
) -> Result<StatusCode> {
    let board = id(&raw)?;
    access::check(&state, who.user_id, board, Access::Editor).await?;
    if body.len() > THUMBNAIL_MAX {
        return Err(Error::too_large("thumbnails are limited to 512 KiB"));
    }
    if !body.starts_with(PNG_MAGIC) {
        return Err(Error::invalid("a thumbnail must be a PNG image"));
    }
    sqlx::query("UPDATE boards SET thumbnail = $2, thumbnail_at = now() WHERE id = $1")
        .bind(board)
        .bind(body.as_ref())
        .execute(&state.pool)
        .await
        .map_err(db)?;
    Ok(StatusCode::NO_CONTENT)
}
