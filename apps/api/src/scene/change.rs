use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use yrs::{Any, GetString, Map, MapRef, Out, Text, TextPrelim, TransactionMut};

use super::read::{self, Shape};

/// A change to one shape: only the fields given change.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Change {
    /// The shape id, from `toile_read_board`.
    pub id: String,
    /// The new label: a sticky's or box's text, a frame's title.
    pub text: Option<String>,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub w: Option<f64>,
    pub h: Option<f64>,
    /// A CSS color: the fill of stickies, boxes and frames, the stroke of text, arrows and lines.
    pub color: Option<String>,
    /// Moves the shape into this frame id; an empty string takes it out of its frame.
    pub frame: Option<String>,
}

fn color(txn: &mut TransactionMut, map: &MapRef, kind: &str, value: &str) {
    let key = if matches!(kind, "text" | "arrow" | "line" | "ink") { "stroke" } else { "fill" };
    let mut style: HashMap<String, Any> = match map.get(txn, "style") {
        Some(Out::Any(Any::Map(old))) => (*old).clone(),
        _ => HashMap::new(),
    };
    style.insert(key.to_owned(), Any::from(value));
    map.insert(txn, "style", Any::from(style));
}

fn label(txn: &mut TransactionMut, map: &MapRef, shape: &Shape, value: &str) -> Result<(), String> {
    match (shape.kind.as_str(), map.get(txn, "text")) {
        ("frame", _) => _ = map.insert(txn, "name", value),
        ("sticky" | "text" | "rect" | "ellipse" | "diamond", Some(Out::YText(text))) => {
            let len = text.get_string(txn).encode_utf16().count() as u32;
            text.remove_range(txn, 0, len);
            text.insert(txn, 0, value);
        }
        ("sticky" | "text" | "rect" | "ellipse" | "diamond", _) => _ = map.insert(txn, "text", TextPrelim::new(value)),
        (kind, _) => return Err(format!("shape {}: a {kind} has no text", shape.id)),
    }
    Ok(())
}

fn frame(txn: &mut TransactionMut, shapes: &MapRef, map: &MapRef, shape: &Shape, to: &str) -> Result<(), String> {
    if to.is_empty() {
        map.remove(txn, "parent");
        return Ok(());
    }
    let target = shapes.get(txn, to).and_then(|value| read::shape(txn, to, &value));
    match target {
        Some(target) if target.kind == "frame" && target.id != shape.id => _ = map.insert(txn, "parent", to),
        _ => return Err(format!("shape {}: {to:?} is not a frame on the board", shape.id)),
    }
    Ok(())
}

fn shift(txn: &mut TransactionMut, shapes: &MapRef, frame: &Shape, (dx, dy): (f64, f64), skip: &[Change]) {
    let children = read::all(txn, shapes).into_iter().filter(|s| s.parent.as_deref() == Some(frame.id.as_str()));
    for child in children.filter(|c| !skip.iter().any(|change| change.id == c.id)).collect::<Vec<_>>() {
        if let Some(Out::YMap(map)) = shapes.get(txn, &child.id) {
            map.insert(txn, "x", child.x + dx);
            map.insert(txn, "y", child.y + dy);
        }
    }
}

fn geometry(txn: &mut TransactionMut, map: &MapRef, change: &Change) {
    for (key, value) in [("x", change.x), ("y", change.y), ("w", change.w), ("h", change.h)] {
        if let Some(value) = value {
            map.insert(txn, key, value);
        }
    }
}

fn one(txn: &mut TransactionMut, shapes: &MapRef, change: &Change, all: &[Change]) -> Result<(), String> {
    let value = shapes.get(txn, &change.id);
    let (Some(Out::YMap(map)), Some(shape)) = (value.clone(), value.and_then(|v| read::shape(txn, &change.id, &v)))
    else {
        return Err(format!("no shape {:?} on the board", change.id));
    };
    geometry(txn, &map, change);
    if shape.kind == "frame" {
        let (dx, dy) = (change.x.unwrap_or(shape.x) - shape.x, change.y.unwrap_or(shape.y) - shape.y);
        shift(txn, shapes, &shape, (dx, dy), all);
    }
    if let Some(text) = &change.text {
        label(txn, &map, &shape, text)?;
    }
    if let Some(value) = &change.color {
        color(txn, &map, &shape.kind, value);
    }
    match &change.frame {
        Some(to) => frame(txn, shapes, &map, &shape, to),
        None => Ok(()),
    }
}

/// Applies every change in one transaction; the first that cannot apply refuses them all.
pub fn apply(txn: &mut TransactionMut, shapes: &MapRef, changes: &[Change]) -> Result<(), String> {
    changes.iter().try_for_each(|change| one(txn, shapes, change, changes))
}
