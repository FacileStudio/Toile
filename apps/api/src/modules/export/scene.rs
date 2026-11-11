use std::collections::HashMap;

use super::geometry::{self, Bounds};
use super::model::{Kind, Shape};

/// Space kept around the drawing.
pub const MARGIN: f64 = 32.0;

const EMPTY: Bounds = Bounds { x0: 0.0, y0: 0.0, x1: 320.0, y1: 200.0 };

/// The shapes of a board arranged for drawing: top-level shapes and frame children, each list
/// in z order, plus the box the drawing covers.
pub struct Scene<'a> {
    pub by_id: HashMap<&'a str, &'a Shape>,
    pub roots: Vec<&'a Shape>,
    pub bounds: Bounds,
    children: HashMap<&'a str, Vec<&'a Shape>>,
}

fn frame_of<'a>(shape: &Shape, by_id: &HashMap<&'a str, &'a Shape>) -> Option<&'a str> {
    let parent = by_id.get(shape.parent.as_deref()?)?;
    (parent.kind == Kind::Frame && parent.id != shape.id).then_some(parent.id.as_str())
}

impl<'a> Scene<'a> {
    /// Arranges `shapes`. With `frame`, only that frame and what it holds; `None` when `frame`
    /// names no frame.
    pub fn new(shapes: &'a [Shape], frame: Option<&str>) -> Option<Scene<'a>> {
        let by_id: HashMap<&str, &Shape> = shapes.iter().map(|s| (s.id.as_str(), s)).collect();
        let mut sorted: Vec<&Shape> = shapes.iter().collect();
        sorted.sort_by(|a, b| (a.z.as_bytes(), a.id.as_str()).cmp(&(b.z.as_bytes(), b.id.as_str())));
        let mut roots = Vec::new();
        let mut children: HashMap<&str, Vec<&Shape>> = HashMap::new();
        for shape in sorted {
            match frame_of(shape, &by_id) {
                Some(parent) => children.entry(parent).or_default().push(shape),
                None => roots.push(shape),
            }
        }
        if let Some(frame) = frame {
            let only = by_id.get(frame).filter(|s| s.kind == Kind::Frame)?;
            roots = vec![*only];
        }
        let bounds = roots.iter().filter_map(|s| geometry::bounds(s, &by_id)).reduce(Bounds::union);
        let bounds = bounds.map_or(EMPTY, |b| b.pad(MARGIN));
        Some(Scene { by_id, roots, bounds, children })
    }

    /// The shapes held by frame `id`, in z order.
    pub fn children(&self, id: &str) -> &[&'a Shape] {
        self.children.get(id).map_or(&[], Vec::as_slice)
    }
}

/// Frame `frame` and every shape it holds, at any depth; `None` when `frame` names no frame.
pub fn within<'a>(shapes: &'a [Shape], frame: &str) -> Option<Vec<&'a Shape>> {
    let by_id: HashMap<&str, &Shape> = shapes.iter().map(|s| (s.id.as_str(), s)).collect();
    by_id.get(frame).filter(|s| s.kind == Kind::Frame)?;
    let inside = |shape: &Shape| {
        let mut at = Some(shape);
        for _ in 0..64 {
            match at {
                Some(s) if s.id == frame => return true,
                Some(s) => at = s.parent.as_deref().and_then(|p| by_id.get(p).copied()),
                None => return false,
            }
        }
        false
    };
    Some(shapes.iter().filter(|s| inside(s)).collect())
}
