mod flow;

use serde::Deserialize;

use crate::scene::input::{Kind, NewShape};

pub use flow::flowchart;

/// The suite's sticky fills, from `@facile/muse` colors.ts `USER_COLORS`.
const FILLS: [&str; 6] = ["#AD9EF0", "#F09ED6", "#EE7E89", "#EEB47E", "#A9EE7E", "#7EEEDB"];
const ROOT_FILL: &str = "#EEB47E";
const ROOT: (f64, f64) = (240.0, 240.0);
const NOTE: (f64, f64) = (200.0, 200.0);
const REACH: f64 = 120.0;
const STEP: f64 = 40.0;
const GAP_Y: f64 = 80.0;
const LEAVES: usize = 6;

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

fn sticky(text: &str, handle: String, color: &str, (x, y): (f64, f64), (w, h): (f64, f64)) -> NewShape {
    let mut shape = NewShape::new(Kind::Sticky, text.trim().chars().take(300).collect::<String>());
    (shape.handle, shape.x, shape.y, shape.w, shape.h) = (Some(handle), Some(x), Some(y), Some(w), Some(h));
    shape.color = Some(color.to_owned());
    shape
}

fn arrow(from: &str, to: &str) -> NewShape {
    let mut shape = NewShape::new(Kind::Arrow, "");
    (shape.from, shape.to) = (Some(from.to_owned()), Some(to.to_owned()));
    shape
}

fn block(branch: &Branch) -> f64 {
    let leaves = branch.children.len().min(LEAVES) as f64;
    NOTE.1.max(leaves * NOTE.1 + (leaves - 1.0).max(0.0) * STEP)
}

fn branch_fill(i: usize) -> &'static str {
    let others: Vec<&'static str> = FILLS.into_iter().filter(|f| *f != ROOT_FILL).collect();
    others[i % others.len()]
}

fn side(out: &mut Vec<NewShape>, branches: &[(usize, &Branch)], right: bool) {
    let total: f64 = branches.iter().map(|(_, b)| block(b)).sum::<f64>() + GAP_Y * (branches.len().max(1) - 1) as f64;
    let (bx, lx) = match right {
        true => (ROOT.0 / 2.0 + REACH, ROOT.0 / 2.0 + 2.0 * REACH + NOTE.0),
        false => (-ROOT.0 / 2.0 - REACH - NOTE.0, -ROOT.0 / 2.0 - 2.0 * (REACH + NOTE.0)),
    };
    let mut top = -total / 2.0;
    for (i, branch) in branches {
        let height = block(branch);
        let (handle, fill) = (format!("b{i}"), branch_fill(*i));
        out.push(sticky(&branch.text, handle.clone(), fill, (bx, top + (height - NOTE.1) / 2.0), NOTE));
        out.push(arrow("root", &handle));
        for (n, leaf) in branch.children.iter().take(LEAVES).enumerate() {
            let leaf_handle = format!("b{i}l{n}");
            let y = top + n as f64 * (NOTE.1 + STEP);
            out.push(sticky(leaf, leaf_handle.clone(), fill, (lx, y), NOTE));
            out.push(arrow(&handle, &leaf_handle));
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

/// A mind map of stickies: the title on a large one, half the branches on each side in their own
/// fill, sub-ideas beyond them in their branch's fill, all joined by arrows; positions start at
/// `(0, 0)`.
pub fn mindmap(map: Mindmap) -> Vec<NewShape> {
    let at = (-ROOT.0 / 2.0, -ROOT.1 / 2.0);
    let mut out = vec![sticky(&map.title, "root".into(), ROOT_FILL, at, ROOT)];
    let branches: Vec<(usize, &Branch)> = map.branches.iter().take(8).enumerate().collect();
    let half = branches.len().div_ceil(2);
    side(&mut out, &branches[..half], true);
    side(&mut out, &branches[half..], false);
    origin(out)
}

/// A titled frame holding one sticky per note, the fills cycling through the suite palette,
/// placed by the board's layout.
pub fn stickies(notes: Stickies) -> Vec<NewShape> {
    let mut frame = NewShape::new(Kind::Frame, notes.title.trim().chars().take(80).collect::<String>());
    frame.handle = Some("frame".into());
    let mut out = vec![frame];
    let notes = notes.stickies.iter().take(30).filter(|n| !n.trim().is_empty());
    for (i, note) in notes.enumerate() {
        let mut sticky = NewShape::new(Kind::Sticky, note.trim().chars().take(300).collect::<String>());
        (sticky.frame, sticky.color) = (Some("frame".into()), Some(FILLS[i % FILLS.len()].to_owned()));
        out.push(sticky);
    }
    out
}

#[cfg(test)]
mod tests;
