use std::collections::{HashMap, HashSet};

use serde::Deserialize;

use crate::scene::input::{Kind, NewShape};

const ROOT: (f64, f64) = (240.0, 120.0);
const BRANCH: (f64, f64) = (220.0, 80.0);
const LEAF: (f64, f64) = (200.0, 60.0);
const REACH: f64 = 120.0;
const STEP: f64 = 20.0;
const GAP_Y: f64 = 40.0;

/// Bulle's mind map: a title and its branches.
#[derive(Debug, Deserialize)]
pub struct Mindmap {
    title: String,
    branches: Vec<Branch>,
}

#[derive(Debug, Deserialize)]
struct Branch {
    text: String,
    children: Vec<String>,
}

/// Bulle's flowchart: nodes and the edges between their ids.
#[derive(Debug, Deserialize)]
pub struct Flowchart {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

#[derive(Debug, Deserialize)]
struct Node {
    id: String,
    text: String,
    kind: String,
}

#[derive(Debug, Deserialize)]
struct Edge {
    from: String,
    to: String,
}

/// Bulle's stickies: a title and one note per idea.
#[derive(Debug, Deserialize)]
pub struct Stickies {
    title: String,
    stickies: Vec<String>,
}

fn boxed(kind: Kind, text: &str, handle: String, (x, y): (f64, f64), (w, h): (f64, f64)) -> NewShape {
    let mut shape = NewShape::new(kind, text.trim().chars().take(300).collect::<String>());
    (shape.handle, shape.x, shape.y, shape.w, shape.h) = (Some(handle), Some(x), Some(y), Some(w), Some(h));
    shape
}

fn joined(kind: Kind, from: &str, to: &str) -> NewShape {
    let mut shape = NewShape::new(kind, "");
    (shape.from, shape.to) = (Some(from.to_owned()), Some(to.to_owned()));
    shape
}

fn block(branch: &Branch) -> f64 {
    let leaves = branch.children.len() as f64;
    BRANCH.1.max(leaves * LEAF.1 + (leaves - 1.0).max(0.0) * STEP)
}

fn side(out: &mut Vec<NewShape>, branches: &[(usize, &Branch)], right: bool) {
    let total: f64 = branches.iter().map(|(_, b)| block(b)).sum::<f64>() + GAP_Y * (branches.len().max(1) - 1) as f64;
    let (bx, lx) = match right {
        true => (ROOT.0 / 2.0 + REACH, ROOT.0 / 2.0 + REACH + BRANCH.0 + REACH),
        false => (-ROOT.0 / 2.0 - REACH - BRANCH.0, -ROOT.0 / 2.0 - 2.0 * REACH - BRANCH.0 - LEAF.0),
    };
    let mut top = -total / 2.0;
    for (i, branch) in branches {
        let height = block(branch);
        let handle = format!("b{i}");
        out.push(boxed(Kind::Rect, &branch.text, handle.clone(), (bx, top + (height - BRANCH.1) / 2.0), BRANCH));
        out.push(joined(Kind::Line, "root", &handle));
        for (n, leaf) in branch.children.iter().enumerate() {
            let leaf_handle = format!("b{i}l{n}");
            let y = top + n as f64 * (LEAF.1 + STEP);
            out.push(boxed(Kind::Rect, leaf, leaf_handle.clone(), (lx, y), LEAF));
            out.push(joined(Kind::Line, &handle, &leaf_handle));
        }
        top += height + GAP_Y;
    }
}

fn origin(mut shapes: Vec<NewShape>) -> Vec<NewShape> {
    let left = shapes.iter().filter_map(|s| s.x).fold(f64::INFINITY, f64::min);
    let top = shapes.iter().filter_map(|s| s.y).fold(f64::INFINITY, f64::min);
    for shape in &mut shapes {
        shape.x = shape.x.map(|x| x - left);
        shape.y = shape.y.map(|y| y - top);
    }
    shapes
}

/// A mind map: the title in an ellipse, half the branches on each side, sub-ideas beyond them,
/// all joined by lines; positions start at `(0, 0)`.
pub fn mindmap(map: Mindmap) -> Vec<NewShape> {
    let mut out = vec![boxed(Kind::Ellipse, &map.title, "root".into(), (-ROOT.0 / 2.0, -ROOT.1 / 2.0), ROOT)];
    let branches: Vec<(usize, &Branch)> = map.branches.iter().take(8).enumerate().collect();
    let half = branches.len().div_ceil(2);
    side(&mut out, &branches[..half], true);
    side(&mut out, &branches[half..], false);
    origin(out)
}

fn layers(nodes: &[&Node], edges: &[(String, String)]) -> HashMap<String, usize> {
    let mut layer: HashMap<String, usize> = nodes.iter().map(|n| (n.id.clone(), 0)).collect();
    for _ in 0..nodes.len() {
        let mut changed = false;
        for (from, to) in edges {
            let next = layer[from] + 1;
            if next < nodes.len() && next > layer[to] {
                layer.insert(to.clone(), next);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    layer
}

fn node_shape(kind: &str) -> (Kind, (f64, f64)) {
    match kind {
        "start" | "end" => (Kind::Ellipse, (200.0, 100.0)),
        "decision" => (Kind::Diamond, (220.0, 140.0)),
        _ => (Kind::Rect, (220.0, 100.0)),
    }
}

/// A flowchart: nodes in rows by their distance from the start, top to bottom, joined by
/// arrows; unknown or repeated ids are dropped. Positions start at `(0, 0)`.
pub fn flowchart(chart: Flowchart) -> Vec<NewShape> {
    let mut seen = HashSet::new();
    let nodes: Vec<&Node> = chart.nodes.iter().take(30).filter(|n| seen.insert(n.id.clone())).collect();
    let edges: Vec<(String, String)> = chart
        .edges
        .iter()
        .take(60)
        .filter(|e| e.from != e.to && seen.contains(&e.from) && seen.contains(&e.to))
        .map(|e| (e.from.clone(), e.to.clone()))
        .collect();
    let layer = layers(&nodes, &edges);
    let mut rows: Vec<Vec<&Node>> = vec![Vec::new(); nodes.len()];
    for node in &nodes {
        rows[layer[&node.id]].push(node);
    }
    let mut out = Vec::new();
    for (depth, row) in rows.iter().enumerate() {
        for (i, node) in row.iter().enumerate() {
            let (kind, (w, h)) = node_shape(&node.kind);
            let center = (i as f64 - (row.len() as f64 - 1.0) / 2.0) * 300.0;
            out.push(boxed(
                kind,
                &node.text,
                format!("n:{}", node.id),
                (center - w / 2.0, depth as f64 * 220.0),
                (w, h),
            ));
        }
    }
    out.extend(edges.iter().map(|(from, to)| joined(Kind::Arrow, &format!("n:{from}"), &format!("n:{to}"))));
    origin(out)
}

/// A titled frame holding one sticky per note, placed by the board's layout.
pub fn stickies(notes: Stickies) -> Vec<NewShape> {
    let mut frame = NewShape::new(Kind::Frame, notes.title.trim().chars().take(80).collect::<String>());
    frame.handle = Some("frame".into());
    let mut out = vec![frame];
    for note in notes.stickies.iter().take(30).filter(|n| !n.trim().is_empty()) {
        let mut sticky = NewShape::new(Kind::Sticky, note.trim().chars().take(300).collect::<String>());
        sticky.frame = Some("frame".into());
        out.push(sticky);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_flowchart_rows_follow_the_edges_and_bad_edges_drop() {
        let chart: Flowchart = serde_json::from_value(json!({
            "nodes": [
                {"id": "s", "text": "Start", "kind": "start"},
                {"id": "d", "text": "Ok?", "kind": "decision"},
                {"id": "e", "text": "Done", "kind": "end"},
                {"id": "s", "text": "again", "kind": "step"}
            ],
            "edges": [{"from": "s", "to": "d"}, {"from": "d", "to": "e"}, {"from": "e", "to": "d"}, {"from": "d", "to": "zz"}]
        }))
        .unwrap();
        let shapes = flowchart(chart);
        let ys: Vec<f64> = shapes.iter().filter_map(|s| s.y).collect();
        assert_eq!(ys, vec![0.0, 220.0, 440.0]);
        assert_eq!(shapes.iter().filter(|s| s.kind == Kind::Arrow).count(), 3);
        assert_eq!(shapes[1].kind, Kind::Diamond);
    }

    #[test]
    fn a_mindmap_puts_branches_on_both_sides_of_the_title() {
        let map: Mindmap = serde_json::from_value(json!({
            "title": "Launch",
            "branches": [{"text": "Site", "children": ["Copy", "Hero"]}, {"text": "Press", "children": []}]
        }))
        .unwrap();
        let shapes = mindmap(map);
        let x = |handle: &str| shapes.iter().find(|s| s.handle.as_deref() == Some(handle)).and_then(|s| s.x).unwrap();
        assert!(x("b0") > x("root") && x("b1") < x("root") && x("b0l0") > x("b0"));
        assert_eq!(shapes.iter().filter_map(|s| s.x).fold(f64::INFINITY, f64::min), 0.0);
        assert_eq!(shapes.iter().filter(|s| s.kind == Kind::Line).count(), 4);
    }
}
