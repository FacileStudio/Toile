use tronc::{Error, Result};

use super::build::{Ids, Imported};
use crate::modules::export::jsoncanvas::{Canvas, Edge, Node};
use crate::modules::export::model::{Binding, Kind, Shape};

const PRESETS: [&str; 6] = ["#fb464c", "#e9973f", "#e0de71", "#44cf6e", "#53dfdd", "#a882ff"];

fn color(raw: Option<&str>) -> Option<String> {
    let raw = raw?;
    match raw.parse::<usize>() {
        Ok(n) if (1..=6).contains(&n) => Some(PRESETS[n - 1].to_owned()),
        _ => raw.starts_with('#').then(|| raw.to_owned()),
    }
}

fn file_name(path: &str) -> String {
    path.rsplit('/').next().unwrap_or(path).to_owned()
}

fn shape(node: &Node, ids: &mut Ids) -> Option<Shape> {
    let kind = match node.kind.as_str() {
        "text" => Kind::Rect,
        "file" | "link" => Kind::Card,
        "group" => Kind::Frame,
        _ => return None,
    };
    let mut shape = Shape::new(ids.get(&node.id), kind);
    (shape.x, shape.y, shape.w, shape.h) = (node.x, node.y, node.width.max(1.0), node.height.max(1.0));
    shape.style.fill = color(node.color.as_deref());
    shape.text = node.text.clone().unwrap_or_default();
    shape.name = node.label.clone().unwrap_or_default();
    if let Some(file) = &node.file {
        (shape.url, shape.title, shape.card_kind) = (file.clone(), file_name(file), "file".to_owned());
    }
    if let Some(url) = &node.url {
        (shape.url, shape.title, shape.card_kind) = (url.clone(), url.clone(), "link".to_owned());
    }
    Some(shape)
}

fn anchor(side: Option<&str>) -> [f64; 2] {
    match side {
        Some("top") => [0.5, 0.0],
        Some("right") => [1.0, 0.5],
        Some("bottom") => [0.5, 1.0],
        Some("left") => [0.0, 0.5],
        _ => [0.5, 0.5],
    }
}

fn arrow(edge: &Edge, shapes: &[Shape], ids: &mut Ids) -> Option<Shape> {
    let end = |id: &str, side: Option<&str>, ids: &mut Ids| {
        let id = ids.get(id);
        let target = shapes.iter().find(|s| s.id == id)?;
        let anchor = anchor(side);
        Some((Binding { shape: id, anchor }, [target.x + anchor[0] * target.w, target.y + anchor[1] * target.h]))
    };
    let (from, start) = end(&edge.from_node, edge.from_side.as_deref(), ids)?;
    let (to, stop) = end(&edge.to_node, edge.to_side.as_deref(), ids)?;
    let mut shape = Shape::new(ids.get(&edge.id), Kind::Arrow);
    (shape.x, shape.y, shape.w, shape.h) = (start[0], start[1], (stop[0] - start[0]).abs(), (stop[1] - start[1]).abs());
    shape.points = vec![0.0, 0.0, stop[0] - start[0], stop[1] - start[1]];
    let arrow_at = |end: Option<&str>, default: &str| end.unwrap_or(default) == "arrow";
    shape.head = Some(
        match (arrow_at(edge.from_end.as_deref(), "none"), arrow_at(edge.to_end.as_deref(), "arrow")) {
            (true, true) => "both",
            (true, false) => "start",
            (false, true) => "end",
            (false, false) => "none",
        }
        .to_owned(),
    );
    shape.style.stroke = color(edge.color.as_deref());
    (shape.from, shape.to) = (Some(from), Some(to));
    Some(shape)
}

fn contains(group: &Shape, shape: &Shape) -> bool {
    group.id != shape.id
        && (shape.kind != Kind::Frame || shape.w * shape.h < group.w * group.h)
        && shape.x >= group.x
        && shape.y >= group.y
        && shape.x + shape.w <= group.x + group.w
        && shape.y + shape.h <= group.y + group.h
}

/// Shapes from a JSON Canvas file: text nodes become rectangles, file and link nodes cards,
/// groups frames holding the nodes inside them, edges bound arrows. Edge labels are dropped.
pub fn import(bytes: &[u8]) -> Result<Imported> {
    let canvas: Canvas =
        serde_json::from_slice(bytes).map_err(|e| Error::invalid(format!("not a JSON Canvas file: {e}")))?;
    let mut ids = Ids::default();
    let mut shapes: Vec<Shape> = canvas.nodes.iter().filter_map(|n| shape(n, &mut ids)).collect();
    shapes.sort_by_key(|s| s.kind != Kind::Frame);
    let groups: Vec<Shape> = shapes.iter().filter(|s| s.kind == Kind::Frame).cloned().collect();
    for shape in &mut shapes {
        let holder = groups.iter().filter(|g| contains(g, shape)).min_by(|a, b| (a.w * a.h).total_cmp(&(b.w * b.h)));
        shape.parent = holder.map(|g| g.id.clone());
    }
    let arrows: Vec<Shape> = canvas.edges.iter().filter_map(|e| arrow(e, &shapes, &mut ids)).collect();
    let labels = canvas.edges.iter().filter(|e| e.label.as_deref().is_some_and(|l| !l.is_empty())).count();
    let skipped = canvas.nodes.len() + canvas.edges.len() - shapes.len() - arrows.len() + labels;
    shapes.extend(arrows);
    Ok(Imported { shapes, skipped })
}
