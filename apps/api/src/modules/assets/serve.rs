use axum::body::Body;
use axum::response::{IntoResponse, Response};
use http::{HeaderMap, HeaderValue, StatusCode, header};
use tronc::{Error, Result};
use uuid::Uuid;

use crate::extract::db;
use crate::state::AppState;

const CACHE: &str = "private, max-age=31536000, immutable";

#[derive(sqlx::FromRow)]
struct Row {
    uploader_id: i64,
    nuage_key: String,
    mime: String,
    size: i64,
    sha256: String,
}

fn fresh(request: &HeaderMap, etag: &str) -> bool {
    request
        .get(header::IF_NONE_MATCH)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.split(',').any(|tag| tag.trim().trim_start_matches("W/") == etag))
}

fn headers(response: &mut Response, etag: &str) {
    let headers = response.headers_mut();
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static(CACHE));
    headers.insert(header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
    if let Ok(value) = HeaderValue::from_str(etag) {
        headers.insert(header::ETAG, value);
    }
}

/// The bytes of `asset` on `board`, read from its uploader's app folder with the uploader's
/// token, so anyone who may see the board sees its images. The caller has checked the board.
pub async fn serve(state: &AppState, board: Uuid, asset: Uuid, request: &HeaderMap) -> Result<Response> {
    let row: Row =
        sqlx::query_as("SELECT uploader_id, nuage_key, mime, size, sha256 FROM assets WHERE id = $1 AND board_id = $2")
            .bind(asset)
            .bind(board)
            .fetch_optional(&state.pool)
            .await
            .map_err(db)?
            .ok_or_else(|| Error::not_found("asset not found"))?;
    let etag = format!("\"{}\"", row.sha256);
    if fresh(request, &etag) {
        let mut response = StatusCode::NOT_MODIFIED.into_response();
        headers(&mut response, &etag);
        return Ok(response);
    }
    let upstream = super::store::get(state, row.uploader_id, &row.nuage_key).await?;
    let mut response = Body::from_stream(upstream.bytes_stream()).into_response();
    headers(&mut response, &etag);
    let content = response.headers_mut();
    content.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&row.mime).unwrap_or(HeaderValue::from_static("image/png")),
    );
    content.insert(header::CONTENT_LENGTH, HeaderValue::from(row.size));
    content.insert(header::CONTENT_DISPOSITION, HeaderValue::from_static("inline"));
    Ok(response)
}
