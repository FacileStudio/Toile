use std::collections::HashMap;

use super::model::{Binding, Kind, Shape};

/// Room left above a frame for its name.
pub const FRAME_LABEL: f64 = 28.0;

/// An axis-aligned box in board coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
}

impl Bounds {
    /// The box around `points`, or `None` when there are none.
    pub fn around(points: impl IntoIterator<Item = [f64; 2]>) -> Option<Bounds> {
        points.into_iter().fold(None, |acc: Option<Bounds>, [x, y]| {
            Some(match acc {
                None => Bounds { x0: x, y0: y, x1: x, y1: y },
                Some(b) => Bounds { x0: b.x0.min(x), y0: b.y0.min(y), x1: b.x1.max(x), y1: b.y1.max(y) },
            })
        })
    }

    /// Both boxes in one.
    pub fn union(self, other: Bounds) -> Bounds {
        Bounds {
            x0: self.x0.min(other.x0),
            y0: self.y0.min(other.y0),
            x1: self.x1.max(other.x1),
            y1: self.y1.max(other.y1),
        }
    }

    /// The box grown by `by` on every side.
    pub fn pad(self, by: f64) -> Bounds {
        Bounds { x0: self.x0 - by, y0: self.y0 - by, x1: self.x1 + by, y1: self.y1 + by }
    }
}

/// `point` turned by `rot` radians around `center`.
pub fn rotate(point: [f64; 2], center: [f64; 2], rot: f64) -> [f64; 2] {
    if rot == 0.0 {
        return point;
    }
    let (sin, cos) = rot.sin_cos();
    let (dx, dy) = (point[0] - center[0], point[1] - center[1]);
    [center[0] + dx * cos - dy * sin, center[1] + dx * sin + dy * cos]
}

/// The board point a binding names: a fraction of the target's box, turned with the target.
pub fn anchor(target: &Shape, anchor: [f64; 2]) -> [f64; 2] {
    let center = [target.x + target.w / 2.0, target.y + target.h / 2.0];
    let point = [target.x + anchor[0] * target.w, target.y + anchor[1] * target.h];
    rotate(point, center, target.rot)
}

fn bound(end: Option<&Binding>, by_id: &HashMap<&str, &Shape>) -> Option<[f64; 2]> {
    let end = end?;
    by_id.get(end.shape.as_str()).map(|target| anchor(target, end.anchor))
}

/// The board points of a line or an arrow, with bound ends moved onto their targets. A binding
/// to a missing shape keeps the stored point.
pub fn path(shape: &Shape, by_id: &HashMap<&str, &Shape>) -> Vec<[f64; 2]> {
    let mut points: Vec<[f64; 2]> =
        shape.points.as_chunks::<2>().0.iter().map(|[x, y]| [shape.x + x, shape.y + y]).collect();
    if points.is_empty() {
        points = vec![[shape.x, shape.y], [shape.x + shape.w, shape.y + shape.h]];
    }
    if points.len() == 1 {
        points.push(points[0]);
    }
    if let Some(start) = bound(shape.from.as_ref(), by_id) {
        points[0] = start;
    }
    if let Some(end) = bound(shape.to.as_ref(), by_id) {
        let last = points.len() - 1;
        points[last] = end;
    }
    points
}

/// The board box a shape covers once drawn, rotation, stroke and frame name included.
pub fn bounds(shape: &Shape, by_id: &HashMap<&str, &Shape>) -> Option<Bounds> {
    let width = shape.style.width.unwrap_or(2.0).max(0.0);
    match shape.kind {
        Kind::Line | Kind::Arrow => Bounds::around(path(shape, by_id)).map(|b| b.pad(width * 3.0 + 8.0)),
        Kind::Ink => {
            let points = shape.points.as_chunks::<3>().0.iter().map(|[x, y, _]| [shape.x + x, shape.y + y]);
            Bounds::around(points).map(|b| b.pad(width * 2.0 + 4.0))
        }
        _ => {
            let center = [shape.x + shape.w / 2.0, shape.y + shape.h / 2.0];
            let top = if shape.kind == Kind::Frame { shape.y - FRAME_LABEL } else { shape.y };
            let corners = [[shape.x, top], [shape.x + shape.w, top], [shape.x, shape.y + shape.h]];
            let corners = corners.into_iter().chain([[shape.x + shape.w, shape.y + shape.h]]);
            Bounds::around(corners.map(|p| rotate(p, center, shape.rot))).map(|b| b.pad(width))
        }
    }
}
