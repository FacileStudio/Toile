use axum::extract::Multipart;
use axum::extract::multipart::Field;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use sha2::{Digest, Sha256};
use tronc::{Error, Result};
use uuid::Uuid;

use super::sniff::{Image, sniff};
use crate::extract::db;
use crate::modules::boards::access::{self, Access};
use crate::modules::suite::{self, UPLOAD_MB};
use crate::state::AppState;

/// The largest upload whatever the plan: Nuage's cap on one object.
pub const MAX_BYTES: usize = 100 << 20;

/// A stored image, as `POST /boards/{id}/assets` answers it.
#[derive(Debug, Serialize)]
pub struct Uploaded {
    asset: Uuid,
    url: String,
    w: u32,
    h: u32,
    mime: &'static str,
    size: usize,
}

/// The plan refusal muse `planLimitFrom` reads: the suite envelope plus `key` and `limit`.
fn plan_limit(limit: i64, upgrade_url: String) -> Response {
    let message = format!("Your plan allows images up to {limit} MB");
    let mut body = tronc::errors::envelope("payment_required", &message, Some(&upgrade_url));
    body["error"]["key"] = UPLOAD_MB.into();
    body["error"]["limit"] = limit.into();
    (http::StatusCode::PAYMENT_REQUIRED, axum::Json(body)).into_response()
}

async fn ceiling(state: &AppState, user_id: i64, space_id: Option<&str>) -> Result<(usize, i64)> {
    let scope = suite::scope(state, user_id, space_id).await?;
    let limit = state.suite.plans.limit(&scope, UPLOAD_MB);
    let bytes = usize::try_from(limit).ok().map_or(MAX_BYTES, |mb| mb.saturating_mul(1 << 20).min(MAX_BYTES));
    Ok((bytes, limit))
}

async fn read(mut field: Field<'_>, cap: usize) -> Result<Option<Vec<u8>>> {
    let mut bytes = Vec::new();
    while let Some(chunk) = field.chunk().await.map_err(|error| Error::invalid(error.body_text()))? {
        if bytes.len() + chunk.len() > cap {
            return Ok(None);
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(Some(bytes))
}

async fn file(multipart: &mut Multipart, cap: usize) -> Result<Option<Vec<u8>>> {
    while let Some(field) = multipart.next_field().await.map_err(|error| Error::invalid(error.body_text()))? {
        if field.name() == Some("file") {
            return read(field, cap).await;
        }
    }
    Err(Error::invalid("the form has no file field"))
}

async fn space_of(state: &AppState, board: Uuid) -> Result<Option<String>> {
    sqlx::query_scalar("SELECT space_id FROM boards WHERE id = $1").bind(board).fetch_one(&state.pool).await.map_err(db)
}

/// Checks `editor` access and the plan's size, sniffs the image, stores it in the uploader's
/// Toile app folder in Nuage and records it on the board.
pub async fn upload(state: &AppState, user_id: i64, board: Uuid, mut multipart: Multipart) -> Result<Response> {
    access::check(state, user_id, board, Access::Editor).await?;
    let space_id = space_of(state, board).await?;
    let (cap, limit) = ceiling(state, user_id, space_id.as_deref()).await?;
    let Some(bytes) = file(&mut multipart, cap).await? else {
        let upgrade = porte::forfait::upgrade_url(&state.issuer, space_id.as_deref().unwrap_or(""));
        return match cap < MAX_BYTES {
            true => Ok(plan_limit(limit, upgrade)),
            false => Err(Error::too_large("images are limited to 100 MB")),
        };
    };
    let image = sniff(&bytes)?;
    let asset = Uuid::new_v4();
    let key = format!("boards/{board}/{asset}.{}", image.ext);
    super::store::put(state, user_id, &key, &image, bytes.clone()).await?;
    record(state, (board, asset, user_id), &key, &image, &bytes).await?;
    let url = format!("/api/boards/{board}/assets/{asset}");
    let answer = Uploaded { asset, url, w: image.width, h: image.height, mime: image.mime, size: bytes.len() };
    Ok((http::StatusCode::CREATED, axum::Json(answer)).into_response())
}

async fn record(state: &AppState, ids: (Uuid, Uuid, i64), key: &str, image: &Image, bytes: &[u8]) -> Result<()> {
    let (board, asset, user_id) = ids;
    let digest: String = Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect();
    sqlx::query(
        "INSERT INTO assets (id, board_id, uploader_id, nuage_key, mime, size, width, height, sha256)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
    )
    .bind(asset)
    .bind(board)
    .bind(user_id)
    .bind(key)
    .bind(image.mime)
    .bind(i64::try_from(bytes.len()).unwrap_or(i64::MAX))
    .bind(i32::try_from(image.width).unwrap_or(i32::MAX))
    .bind(i32::try_from(image.height).unwrap_or(i32::MAX))
    .bind(digest)
    .execute(&state.pool)
    .await
    .map_err(db)?;
    Ok(())
}
