use tronc::{Error, Result};

use super::sniff::Image;
use crate::state::AppState;

fn object_url(state: &AppState) -> String {
    format!("{}/api/app/object", state.suite.apps.nuage.url)
}

fn refused(status: reqwest::StatusCode) -> Error {
    match status.as_u16() {
        402 | 413 | 507 => Error::payment_required("your Nuage storage is full"),
        _ => {
            tracing::warn!(status = status.as_u16(), "assets: nuage refused the upload");
            Error::unavailable("Nuage could not store the image")
        }
    }
}

/// Stores `bytes` in the uploader's Toile app folder under `key`, named after its last segment.
pub async fn put(state: &AppState, user_id: i64, key: &str, image: &Image, bytes: Vec<u8>) -> Result<()> {
    let name = key.rsplit('/').next().unwrap_or(key);
    let url = object_url(state);
    let body = bytes::Bytes::from(bytes);
    let response = state
        .suite
        .send(user_id, &state.suite.apps.nuage, true, |http, token| {
            http.put(&url)
                .query(&[("key", key), ("name", name)])
                .bearer_auth(token)
                .header(http::header::CONTENT_TYPE, image.mime)
                .body(body.clone())
        })
        .await?;
    match response.status().is_success() {
        true => Ok(()),
        false => Err(refused(response.status())),
    }
}

/// Opens the object `key` of `uploader`'s app folder, streamed.
pub async fn get(state: &AppState, uploader: i64, key: &str) -> Result<reqwest::Response> {
    let url = object_url(state);
    let response = state
        .suite
        .send(uploader, &state.suite.apps.nuage, true, |http, token| {
            http.get(&url).query(&[("key", key)]).bearer_auth(token)
        })
        .await
        .map_err(|error| {
            tracing::warn!(error = %error.message, "assets: no token for the uploader");
            Error::unavailable("the image is unavailable until its uploader signs in to Facile again")
        })?;
    match response.status().is_success() {
        true => Ok(response),
        false if response.status() == reqwest::StatusCode::NOT_FOUND => Err(Error::not_found("asset not found")),
        false => Err(Error::unavailable("Nuage could not serve the image")),
    }
}
