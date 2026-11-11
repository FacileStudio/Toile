use std::collections::HashMap;

use super::input::{Kind, NewShape, Placed, Planned};
use super::layout::{FRAME_TOP, GAP, ROW, Shelf, after, bounds, default_size};
use super::read::Shape;

/// The most shapes one call adds.
pub const MAX_NEW: usize = 500;

struct Plan<'a> {
    existing: &'a [Shape],
    handles: HashMap<String, String>,
    boxes: HashMap<String, (bool, f64, f64, f64, f64)>,
    floating: HashMap<String, usize>,
    shelves: HashMap<String, Shelf>,
    main: Shelf,
    z: String,
}

impl<'a> Plan<'a> {
    fn new(existing: &'a [Shape], input: &[NewShape]) -> Plan<'a> {
        let main = match bounds(existing) {
            Some((_, top, right, _)) => Shelf::new(right + 4.0 * GAP, top, ROW),
            None => Shelf::new(0.0, 0.0, ROW),
        };
        let mut floating = HashMap::new();
        for frame in input.iter().filter(|s| s.floating()).filter_map(|s| s.frame.clone()) {
            *floating.entry(frame).or_insert(0) += 1;
        }
        let z = existing.iter().map(|s| s.z.as_str()).max().map_or_else(String::new, str::to_owned);
        let boxes = existing.iter().map(|s| (s.id.clone(), (s.kind == "frame", s.x, s.y, s.w, s.h))).collect();
        Plan { existing, handles: HashMap::new(), boxes, floating, shelves: HashMap::new(), main, z }
    }

    fn resolve(&self, name: &str, frame: bool) -> Result<String, String> {
        let id = self.handles.get(name).cloned().unwrap_or_else(|| name.to_owned());
        match self.boxes.get(&id) {
            None => Err(format!("no shape or ref {name:?} on the board or earlier in this call")),
            Some(found) if frame && !found.0 => Err(format!("{name:?} is not a frame")),
            Some(_) => Ok(id),
        }
    }

    fn frame_size(&self, shape: &NewShape) -> (f64, f64) {
        let count = shape.handle.as_ref().and_then(|h| self.floating.get(h)).copied().unwrap_or(0);
        if count == 0 {
            return default_size("frame");
        }
        let cols = (count as f64).sqrt().ceil();
        let rows = (count as f64 / cols).ceil();
        (GAP + cols * (200.0 + GAP), FRAME_TOP + rows * (200.0 + GAP))
    }

    fn shelf(&mut self, parent: Option<&str>) -> &mut Shelf {
        let Some(frame) = parent else { return &mut self.main };
        let (_, x, y, w, _) = self.boxes[frame];
        let below = self.existing.iter().filter(|s| s.parent.as_deref() == Some(frame)).map(|s| s.y + s.h + GAP);
        let top = below.fold(y + FRAME_TOP, f64::max);
        self.shelves.entry(frame.to_owned()).or_insert_with(|| Shelf::new(x + GAP, top, w - 2.0 * GAP))
    }

    fn place_box(&mut self, shape: NewShape, id: String) -> Result<Planned, String> {
        let (dw, dh) = match shape.kind {
            Kind::Frame => self.frame_size(&shape),
            kind => default_size(kind.name()),
        };
        let (w, h) = (shape.w.unwrap_or(dw), shape.h.unwrap_or(dh));
        let parent = shape.frame.as_deref().map(|f| self.resolve(f, true)).transpose()?;
        let (x, y) = match (shape.x, shape.y) {
            (Some(x), Some(y)) => (x, y),
            _ => self.shelf(parent.as_deref()).place(w, h),
        };
        self.boxes.insert(id.clone(), (shape.kind == Kind::Frame, x, y, w, h));
        Ok(Planned::new(
            Placed { id, handle: shape.handle, kind: shape.kind, x, y, w, h },
            &shape.text,
            shape.color,
            parent,
        ))
    }

    fn end(&self, name: Option<&str>, fallback: Option<(f64, f64)>) -> Result<(Option<String>, (f64, f64)), String> {
        if let Some(name) = name {
            let id = self.resolve(name, false)?;
            let (_, x, y, w, h) = self.boxes[&id];
            return Ok((Some(id), (x + w / 2.0, y + h / 2.0)));
        }
        fallback.map(|at| (None, at)).ok_or_else(|| "an arrow or line needs from and to, or x, y, w and h".to_owned())
    }
}

fn place_path(plan: &Plan, shape: NewShape, id: String) -> Result<Planned, String> {
    let start = shape.x.zip(shape.y);
    let stop = start.map(|(x, y)| (x + shape.w.unwrap_or(0.0), y + shape.h.unwrap_or(0.0)));
    let (from, (x, y)) = plan.end(shape.from.as_deref(), start)?;
    let (to, (ex, ey)) = plan.end(shape.to.as_deref(), stop)?;
    let placed = Placed { id, handle: shape.handle, kind: shape.kind, x, y, w: (ex - x).abs(), h: (ey - y).abs() };
    Ok(Planned { from, to, end: (ex - x, ey - y), ..Planned::new(placed, &None, shape.color, None) })
}

/// Plans `input` on a board holding `existing`: ids from `fresh`, frames first, positions for
/// the shapes that have none, arrows bound to the shapes they name, and paint order on top.
pub fn plan(
    existing: &[Shape],
    mut input: Vec<NewShape>,
    mut fresh: impl FnMut() -> String,
) -> Result<Vec<Planned>, String> {
    if input.len() > MAX_NEW {
        return Err(format!("add at most {MAX_NEW} shapes per call"));
    }
    input.sort_by_key(NewShape::rank);
    let mut plan = Plan::new(existing, &input);
    let mut out = Vec::with_capacity(input.len());
    for shape in input {
        let id = fresh();
        if let Some(handle) = &shape.handle
            && plan.handles.insert(handle.clone(), id.clone()).is_some()
        {
            return Err(format!("ref {handle:?} is used twice"));
        }
        let mut next = match shape.kind.is_path() {
            true => place_path(&plan, shape, id)?,
            false => plan.place_box(shape, id)?,
        };
        plan.z = after(&plan.z);
        next.z = plan.z.clone();
        out.push(next);
    }
    Ok(out)
}
