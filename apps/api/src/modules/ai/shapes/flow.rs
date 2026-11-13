use std::collections::{HashMap, HashSet};

use super::{Flowchart, NOTE, Node, arrow, origin, sticky};
use crate::scene::input::NewShape;

const ENDS: &str = "#A9EE7E";
const STEP_FILL: &str = "#AD9EF0";
const DECISION: &str = "#EE7E89";
const ROW: f64 = 300.0;
const COLUMN: f64 = 280.0;

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

fn hex(color: &str) -> bool {
    color.len() == 7 && color.starts_with('#') && color[1..].chars().all(|c| c.is_ascii_hexdigit())
}

fn fill<'a>(kind: &str, decision: &'a str) -> &'a str {
    match kind {
        "start" | "end" => ENDS,
        "decision" => decision,
        _ => STEP_FILL,
    }
}

/// A flowchart of stickies: nodes in rows by their distance from the start, top to bottom,
/// joined by arrows. Starts and ends are green, steps violet, decisions take `accent` when it is
/// a `#rrggbb` color and red otherwise. Unknown or repeated ids are dropped. Positions start at
/// `(0, 0)`.
pub fn flowchart(chart: Flowchart, accent: Option<&str>) -> Vec<NewShape> {
    let decision = accent.filter(|a| hex(a)).unwrap_or(DECISION);
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
            let center = (i as f64 - (row.len() as f64 - 1.0) / 2.0) * COLUMN;
            let at = (center - NOTE.0 / 2.0, depth as f64 * ROW);
            out.push(sticky(&node.text, format!("n:{}", node.id), fill(&node.kind, decision), at, NOTE));
        }
    }
    out.extend(edges.iter().map(|(from, to)| arrow(&format!("n:{from}"), &format!("n:{to}"))));
    origin(out)
}
