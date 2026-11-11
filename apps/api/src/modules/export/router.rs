use axum::Router;
use axum::extract::{Path, State};
use axum::http::HeaderValue;
use axum::response::Response;
use axum::routing::get;
use porte::Identity;
use serde::Deserialize;
use tronc::{Error, Result};

use super::raster::{self, MAX_SIDE};
use super::respond::{asset_base, blocking, draw, file};
use super::{jsoncanvas, read, scene, source};
use crate::extract::{Params, id};
use crate::state::AppState;

const WHITE: Option<&str> = Some("#ffffff");

/// `?frame=`: export one frame and what it holds instead of the whole board.
#[derive(Debug, Default, Deserialize)]
struct FrameQuery {
    frame: Option<String>,
}

async fn svg(
    State(state): State<AppState>,
    who: Identity,
    Path(raw): Path<String>,
    Params(q): Params<FrameQuery>,
) -> Result<Response> {
    let (board, doc) = source::board(&state, who.user_id, id(&raw)?).await?;
    let assets = asset_base(board.id);
    let drawing = draw(&doc, q.frame.as_deref(), None, &assets)?;
    Ok(file(drawing.svg, "image/svg+xml", &board.name, "svg"))
}

async fn png(
    State(state): State<AppState>,
    who: Identity,
    Path(raw): Path<String>,
    Params(q): Params<FrameQuery>,
) -> Result<Response> {
    let (board, doc) = source::board(&state, who.user_id, id(&raw)?).await?;
    let assets = asset_base(board.id);
    let drawing = draw(&doc, q.frame.as_deref(), WHITE, &assets)?;
    let bytes = blocking(move || raster::png(&drawing, 2.0, MAX_SIDE)).await?;
    Ok(file(bytes, "image/png", &board.name, "png"))
}

async fn pdf(
    State(state): State<AppState>,
    who: Identity,
    Path(raw): Path<String>,
    Params(q): Params<FrameQuery>,
) -> Result<Response> {
    let (board, doc) = source::board(&state, who.user_id, id(&raw)?).await?;
    let assets = asset_base(board.id);
    let drawing = draw(&doc, q.frame.as_deref(), WHITE, &assets)?;
    let bytes = blocking(move || raster::pdf(&drawing)).await?;
    Ok(file(bytes, "application/pdf", &board.name, "pdf"))
}

async fn canvas(
    State(state): State<AppState>,
    who: Identity,
    Path(raw): Path<String>,
    Params(q): Params<FrameQuery>,
) -> Result<Response> {
    let (board, doc) = source::board(&state, who.user_id, id(&raw)?).await?;
    let assets = asset_base(board.id);
    let shapes = read::shapes(&doc);
    let picked = match q.frame.as_deref() {
        Some(frame) => scene::within(&shapes, frame).ok_or_else(|| Error::not_found("frame not found"))?,
        None => shapes.iter().collect(),
    };
    let (canvas, skipped) = jsoncanvas::export(&picked, &assets);
    let body = serde_json::to_vec(&canvas).map_err(|e| Error::internal("canvas encoding failed", e))?;
    let mut response = file(body, "application/json", &board.name, "canvas");
    response.headers_mut().insert("x-toile-skipped", HeaderValue::from(skipped));
    Ok(response)
}

/// The export routes, mounted under `/api` behind `require_auth`.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/boards/{id}/export.svg", get(svg))
        .route("/boards/{id}/export.png", get(png))
        .route("/boards/{id}/export.pdf", get(pdf))
        .route("/boards/{id}/export.canvas", get(canvas))
}
