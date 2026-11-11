use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use super::model::{Binding, Kind, Shape};

/// A JSON Canvas 1.0 document (<https://jsoncanvas.org/spec/1.0/>).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Canvas {
    #[serde(default)]
    pub nodes: Vec<Node>,
    #[serde(default)]
    pub edges: Vec<Edge>,
}

/// A JSON Canvas node: `text`, `file`, `link` or `group`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Node {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(serialize_with = "whole")]
    pub x: f64,
    #[serde(serialize_with = "whole")]
    pub y: f64,
    #[serde(serialize_with = "whole")]
    pub width: f64,
    #[serde(serialize_with = "whole")]
    pub height: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// A JSON Canvas edge between two nodes.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Edge {
    pub id: String,
    pub from_node: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_side: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_end: Option<String>,
    pub to_node: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to_side: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to_end: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

fn whole<S: serde::Serializer>(value: &f64, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_i64(value.round() as i64)
}

fn hex(raw: Option<&str>) -> Option<String> {
    raw.filter(|c| c.len() == 7 && c.starts_with('#') && c[1..].chars().all(|d| d.is_ascii_hexdigit()))
        .map(str::to_owned)
}

fn node(shape: &Shape, asset_base: &str) -> Option<Node> {
    let mut node = Node {
        id: shape.id.clone(),
        x: shape.x.round(),
        y: shape.y.round(),
        width: shape.w.round().max(1.0),
        height: shape.h.round().max(1.0),
        color: hex(shape.style.fill.as_deref()),
        ..Node::default()
    };
    match shape.kind {
        k if k.has_text() => (node.kind, node.text) = ("text".into(), Some(shape.text.clone())),
        Kind::Card if shape.card_kind == "file" => (node.kind, node.file) = ("file".into(), Some(shape.url.clone())),
        Kind::Card => (node.kind, node.url) = ("link".into(), Some(shape.url.clone())),
        Kind::Image => (node.kind, node.file) = ("file".into(), Some(format!("{asset_base}{}", shape.asset))),
        Kind::Frame => (node.kind, node.label) = ("group".into(), Some(shape.name.clone())),
        _ => return None,
    }
    Some(node)
}

/// The JSON Canvas side an anchor is closest to.
pub fn side(anchor: [f64; 2]) -> &'static str {
    let [u, v] = anchor;
    let sides = [(v, "top"), (1.0 - u, "right"), (1.0 - v, "bottom"), (u, "left")];
    sides.iter().fold((f64::MAX, "top"), |best, (d, s)| if *d < best.0 { (*d, *s) } else { best }).1
}

fn end(heads: &str, at: &str) -> String {
    if heads == at || heads == "both" { "arrow".into() } else { "none".into() }
}

fn edge(shape: &Shape, nodes: &HashSet<&str>) -> Option<Edge> {
    let bound = |b: Option<&Binding>| b.filter(|b| nodes.contains(b.shape.as_str())).cloned();
    let (from, to) = (bound(shape.from.as_ref())?, bound(shape.to.as_ref())?);
    let heads = shape.head.as_deref().unwrap_or(if shape.kind == Kind::Arrow { "end" } else { "none" });
    Some(Edge {
        id: shape.id.clone(),
        from_node: from.shape,
        from_side: Some(side(from.anchor).into()),
        from_end: Some(end(heads, "start")),
        to_node: to.shape,
        to_side: Some(side(to.anchor).into()),
        to_end: Some(end(heads, "end")),
        color: hex(shape.style.stroke.as_deref()),
        label: None,
    })
}

/// `shapes` as JSON Canvas, ordered by z, and how many shapes had no equivalent (ink, and lines
/// or arrows not bound at both ends).
pub fn export(shapes: &[&Shape], asset_base: &str) -> (Canvas, usize) {
    let mut sorted: Vec<&Shape> = shapes.to_vec();
    sorted.sort_by(|a, b| (a.z.as_bytes(), a.id.as_str()).cmp(&(b.z.as_bytes(), b.id.as_str())));
    let nodes: Vec<Node> = sorted.iter().filter_map(|s| node(s, asset_base)).collect();
    let ids: HashSet<&str> = nodes.iter().map(|n| n.id.as_str()).collect();
    let connectors = sorted.iter().filter(|s| matches!(s.kind, Kind::Arrow | Kind::Line));
    let edges: Vec<Edge> = connectors.filter_map(|s| edge(s, &ids)).collect();
    let skipped = shapes.len() - nodes.len() - edges.len();
    (Canvas { nodes, edges }, skipped)
}
