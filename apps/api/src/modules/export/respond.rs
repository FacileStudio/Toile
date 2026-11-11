use axum::body::Body;
use axum::http::{HeaderValue, header};
use axum::response::Response;
use tronc::{Error, Result};
use uuid::Uuid;
use yrs::Doc;

use super::read;
use super::svg::{self, Drawing, Options};

/// Where the images of board `board` are served from, relative to the app's origin: an image
/// shape's `asset` id is appended to it.
pub fn asset_base(board: Uuid) -> String {
    format!("/api/boards/{board}/assets/")
}

/// `bytes` as a download named after `title`.
pub fn file(bytes: impl Into<Body>, mime: &'static str, title: &str, extension: &str) -> Response {
    let name: String = title.chars().filter(|c| c.is_alphanumeric() || " -_".contains(*c)).collect();
    let name = if name.trim().is_empty() { "board".to_owned() } else { name.trim().to_owned() };
    let mut response = Response::new(bytes.into());
    let headers = response.headers_mut();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(mime));
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("private, no-cache"));
    let disposition = format!("attachment; filename=\"{name}.{extension}\"");
    if let Ok(value) = HeaderValue::from_str(&disposition) {
        headers.insert(header::CONTENT_DISPOSITION, value);
    }
    response
}

/// `doc` drawn as SVG; `404` when `frame` names no frame.
pub fn draw(doc: &Doc, frame: Option<&str>, background: Option<&str>, assets: &str) -> Result<Drawing> {
    let shapes = read::shapes(doc);
    let options = Options { frame, background, asset_base: assets };
    svg::render(&shapes, &options).ok_or_else(|| Error::not_found("frame not found"))
}

/// Runs a CPU-bound render off the async workers.
pub async fn blocking<T: Send + 'static>(work: impl FnOnce() -> Result<T> + Send + 'static) -> Result<T> {
    tokio::task::spawn_blocking(work).await.map_err(|e| Error::internal("render task failed", e))?
}
