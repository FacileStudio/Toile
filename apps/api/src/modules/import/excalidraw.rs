use std::collections::HashMap;

use serde::Deserialize;
use tronc::{Error, Result};

use super::build::{Ids, Imported};
use super::elements;
use crate::modules::export::model::Kind;

/// An `.excalidraw` file. Only `elements` matters; app state and embedded files are ignored.
#[derive(Debug, Deserialize)]
pub struct File {
    #[serde(rename = "type")]
    pub kind: Option<String>,
    pub elements: Vec<Element>,
}

/// An arrow end bound to an element.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Bind {
    pub element_id: String,
    pub fixed_point: Option<[f64; 2]>,
}

/// One Excalidraw element, with the fields the import reads.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Element {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub angle: f64,
    pub stroke_color: Option<String>,
    pub background_color: Option<String>,
    pub stroke_width: Option<f64>,
    pub font_size: Option<f64>,
    pub font_family: Option<f64>,
    pub text: Option<String>,
    pub text_align: Option<String>,
    pub container_id: Option<String>,
    pub points: Vec<[f64; 2]>,
    pub pressures: Vec<f64>,
    pub start_binding: Option<Bind>,
    pub end_binding: Option<Bind>,
    pub start_arrowhead: Option<String>,
    pub end_arrowhead: Option<String>,
    pub frame_id: Option<String>,
    pub name: Option<String>,
    pub is_deleted: bool,
}

/// The schema-1 type an element becomes, or `None` when it has no equivalent.
pub fn kind(element: &Element) -> Option<Kind> {
    Some(match element.kind.as_str() {
        "rectangle" => Kind::Rect,
        "ellipse" => Kind::Ellipse,
        "diamond" => Kind::Diamond,
        "text" => Kind::Text,
        "arrow" => Kind::Arrow,
        "line" => Kind::Line,
        "freedraw" => Kind::Ink,
        "frame" | "magicframe" => Kind::Frame,
        _ => return None,
    })
}

/// Shapes from an Excalidraw scene: rectangles, ellipses, diamonds, text (text bound to a
/// container becomes the container's text), arrows with their bindings, lines, freedraw as
/// ink, and frames. Images, embeds and arrow labels are counted as skipped.
pub fn import(bytes: &[u8]) -> Result<Imported> {
    let file: File =
        serde_json::from_slice(bytes).map_err(|e| Error::invalid(format!("not an Excalidraw file: {e}")))?;
    if file.kind.as_deref().is_some_and(|k| k != "excalidraw") {
        return Err(Error::invalid("not an Excalidraw file"));
    }
    let live: Vec<&Element> = file.elements.iter().filter(|e| !e.is_deleted).collect();
    let by_id: HashMap<&str, &Element> = live.iter().map(|e| (e.id.as_str(), *e)).collect();
    let mut ids = Ids::default();
    let mut imported = Imported::default();
    for element in &live {
        match elements::shape(element, &by_id, &mut ids) {
            elements::Outcome::Shape(shape) => imported.shapes.push(*shape),
            elements::Outcome::Merged => {}
            elements::Outcome::Skipped => imported.skipped += 1,
        }
    }
    elements::attach_text(&live, &mut imported, &mut ids);
    Ok(imported)
}
