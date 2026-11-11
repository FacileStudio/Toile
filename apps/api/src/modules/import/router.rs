use axum::Router;
use axum::extract::multipart::{Field, MultipartRejection};
use axum::extract::{DefaultBodyLimit, Multipart, State};
use axum::http::StatusCode;
use axum::routing::post;
use porte::Identity;
use tronc::json::Json;
use tronc::{Error, Result};

use super::service::{self, Answer, MAX_FILE, Upload};
use crate::state::AppState;

fn bad(error: impl std::fmt::Display) -> Error {
    Error::invalid(format!("unreadable upload: {error}"))
}

async fn bytes(mut field: Field<'_>, cap: usize) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    while let Some(chunk) = field.chunk().await.map_err(bad)? {
        if out.len() + chunk.len() > cap {
            return Err(Error::too_large(format!("the file is larger than {} MiB", cap / 1024 / 1024)));
        }
        out.extend_from_slice(&chunk);
    }
    Ok(out)
}

async fn text(field: Field<'_>) -> Result<String> {
    String::from_utf8(bytes(field, 4096).await?).map_err(bad)
}

async fn upload(mut multipart: Multipart) -> Result<Upload> {
    let mut upload = Upload::default();
    while let Some(field) = multipart.next_field().await.map_err(bad)? {
        match field.name().unwrap_or_default() {
            "file" => {
                upload.file_name = field.file_name().unwrap_or_default().to_owned();
                upload.bytes = bytes(field, MAX_FILE).await?;
            }
            "name" => upload.name = Some(text(field).await?),
            "space_id" => upload.space_id = Some(text(field).await?).filter(|s| !s.is_empty()),
            other => return Err(Error::invalid(format!("unknown field {other}"))),
        }
    }
    if upload.bytes.is_empty() {
        return Err(Error::invalid("the file field is missing or empty"));
    }
    Ok(upload)
}

async fn import(
    State(state): State<AppState>,
    who: Identity,
    multipart: std::result::Result<Multipart, MultipartRejection>,
) -> Result<(StatusCode, Json<Answer>)> {
    let upload = upload(multipart.map_err(bad)?).await?;
    Ok((StatusCode::CREATED, Json(service::import(&state, who.user_id, upload).await?)))
}

/// The import route, mounted under `/api` behind `require_auth`.
pub fn routes() -> Router<AppState> {
    Router::new().route("/boards/import", post(import).layer(DefaultBodyLimit::max(MAX_FILE + 64 * 1024)))
}
