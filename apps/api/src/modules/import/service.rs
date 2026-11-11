use std::collections::HashSet;

use serde::Serialize;
use tronc::{Error, Result};

use super::build::{self, Imported};
use super::{excalidraw, jsoncanvas};
use crate::modules::board::schema;
use crate::modules::boards::service;
use crate::modules::boards::types::{Board, CreateRequest};
use crate::modules::export::model::{Kind, Shape};
use crate::state::AppState;

/// The largest file an import accepts.
pub const MAX_FILE: usize = 5 * 1024 * 1024;

/// An uploaded board file and where it should land.
#[derive(Debug, Default)]
pub struct Upload {
    pub file_name: String,
    pub bytes: Vec<u8>,
    pub name: Option<String>,
    pub space_id: Option<String>,
}

/// The board an import created, and how many source elements had no equivalent.
#[derive(Debug, Serialize)]
pub struct Answer {
    #[serde(flatten)]
    pub board: Board,
    pub skipped: usize,
}

/// Which importer reads `upload`: by extension, else by the file's top-level keys.
pub fn parse(upload: &Upload) -> Result<Imported> {
    let lower = upload.file_name.to_lowercase();
    if lower.ends_with(".canvas") {
        return jsoncanvas::import(&upload.bytes);
    }
    if lower.ends_with(".excalidraw") {
        return excalidraw::import(&upload.bytes);
    }
    let value: serde_json::Value =
        serde_json::from_slice(&upload.bytes).map_err(|_| Error::invalid("expected a .canvas or .excalidraw file"))?;
    match (value.get("elements"), value.get("nodes")) {
        (Some(_), _) => excalidraw::import(&upload.bytes),
        (None, Some(_)) => jsoncanvas::import(&upload.bytes),
        _ => Err(Error::invalid("expected a .canvas or .excalidraw file")),
    }
}

/// Drops what points at nothing: bindings to a missing shape, parents that are not frames.
pub fn prune(shapes: &mut [Shape]) {
    let frames: HashSet<String> = shapes.iter().filter(|s| s.kind == Kind::Frame).map(|s| s.id.clone()).collect();
    let all: HashSet<String> = shapes.iter().map(|s| s.id.clone()).collect();
    for shape in shapes.iter_mut() {
        shape.parent = shape.parent.take().filter(|p| frames.contains(p) && *p != shape.id);
        shape.from = shape.from.take().filter(|b| all.contains(&b.shape) && b.shape != shape.id);
        shape.to = shape.to.take().filter(|b| all.contains(&b.shape) && b.shape != shape.id);
    }
}

fn stem(file_name: &str) -> String {
    let base = file_name.rsplit(['/', '\\']).next().unwrap_or(file_name);
    base.rsplit_once('.').map_or(base, |(stem, _)| stem).to_owned()
}

/// Creates a board in the upload's scope holding what the file describes. The shapes are
/// checked against schema 1 first, then written through the board's room.
pub async fn import(state: &AppState, user_id: i64, upload: Upload) -> Result<Answer> {
    let mut imported = parse(&upload)?;
    prune(&mut imported.shapes);
    schema::check_all(&build::document(&imported.shapes)).map_err(Error::invalid)?;
    let name = upload.name.unwrap_or_else(|| stem(&upload.file_name));
    let board = service::create(state, user_id, CreateRequest { name, space_id: upload.space_id }).await?;
    let shapes = imported.shapes;
    let edit = Box::new(move |txn: &mut yrs::TransactionMut, root: &yrs::MapRef| {
        build::fill(txn, root, &shapes);
        Ok(())
    });
    state.hub.write(state, board.id, edit).await?;
    Ok(Answer { board, skipped: imported.skipped })
}
