use std::collections::HashMap;

use super::build::{Ids, Imported};
use super::excalidraw::{Bind, Element, kind};
use crate::modules::export::model::{Binding, Kind, Shape, Style};

/// The most ink points schema 1 keeps per stroke.
pub const MAX_INK_POINTS: usize = 10_000;

/// The most characters schema 1 keeps per text.
pub const MAX_TEXT: usize = 10_000;

/// What became of one element.
pub enum Outcome {
    Shape(Box<Shape>),
    Merged,
    Skipped,
}

fn style(element: &Element) -> Style {
    let mono = matches!(element.font_family.map(|f| f as i64), Some(3 | 8));
    Style {
        fill: element.background_color.clone().filter(|c| c != "transparent"),
        stroke: element.stroke_color.clone(),
        width: element.stroke_width,
        font: mono.then(|| "mono".to_owned()),
        size: element.font_size,
        align: element.text_align.clone(),
    }
}

fn binding(bind: Option<&Bind>, end: [f64; 2], by_id: &HashMap<&str, &Element>, ids: &mut Ids) -> Option<Binding> {
    let bind = bind?;
    let target = by_id.get(bind.element_id.as_str()).filter(|t| kind(t).is_some_and(|k| k != Kind::Arrow))?;
    let ratio = |v: f64, from: f64, size: f64| if size > 0.0 { (v - from) / size } else { 0.5 };
    let anchor =
        bind.fixed_point.unwrap_or([ratio(end[0], target.x, target.width), ratio(end[1], target.y, target.height)]);
    Some(Binding { shape: ids.get(&target.id), anchor: [anchor[0].clamp(0.0, 1.0), anchor[1].clamp(0.0, 1.0)] })
}

fn connector(element: &Element, shape: &mut Shape, by_id: &HashMap<&str, &Element>, ids: &mut Ids) {
    shape.points = element.points.iter().flat_map(|p| *p).collect();
    if shape.kind != Kind::Arrow {
        return;
    }
    let absolute = |p: Option<&[f64; 2]>| p.map_or([element.x, element.y], |p| [element.x + p[0], element.y + p[1]]);
    shape.from = binding(element.start_binding.as_ref(), absolute(element.points.first()), by_id, ids);
    shape.to = binding(element.end_binding.as_ref(), absolute(element.points.last()), by_id, ids);
    let head = (element.start_arrowhead.is_some(), element.end_arrowhead.is_some());
    shape.head = Some(
        match head {
            (true, true) => "both",
            (true, false) => "start",
            (false, true) => "end",
            (false, false) => "none",
        }
        .to_owned(),
    );
}

fn ink(element: &Element) -> Vec<f64> {
    let step = element.points.len().div_ceil(MAX_INK_POINTS).max(1);
    let pressure = |i: usize| element.pressures.get(i).copied().filter(|p| *p > 0.0).unwrap_or(0.5);
    element.points.iter().enumerate().step_by(step).flat_map(|(i, p)| [p[0], p[1], pressure(i)]).collect()
}

fn bound_text(element: &Element, by_id: &HashMap<&str, &Element>) -> Option<bool> {
    let container = element.container_id.as_deref()?;
    let holder = by_id.get(container).and_then(|c| kind(c));
    Some(holder.is_some_and(Kind::has_text))
}

/// The shape `element` becomes. Text bound to a container is `Merged` here and attached by
/// [`attach_text`].
pub fn shape(element: &Element, by_id: &HashMap<&str, &Element>, ids: &mut Ids) -> Outcome {
    let Some(shape_kind) = kind(element) else { return Outcome::Skipped };
    match bound_text(element, by_id) {
        Some(true) => return Outcome::Merged,
        Some(false) => return Outcome::Skipped,
        None => {}
    }
    let mut shape = Shape::new(ids.get(&element.id), shape_kind);
    (shape.x, shape.y, shape.w, shape.h) = (element.x, element.y, element.width.abs(), element.height.abs());
    (shape.rot, shape.style) = (element.angle, style(element));
    shape.text = element.text.clone().unwrap_or_default().chars().take(MAX_TEXT).collect();
    shape.name = element.name.clone().unwrap_or_else(|| "Frame".to_owned());
    let frame = element.frame_id.as_deref().and_then(|f| by_id.get(f)).filter(|f| kind(f) == Some(Kind::Frame));
    shape.parent = frame.map(|f| ids.get(&f.id));
    match shape_kind {
        Kind::Arrow | Kind::Line => connector(element, &mut shape, by_id, ids),
        Kind::Ink => shape.points = ink(element),
        _ => {}
    }
    Outcome::Shape(Box::new(shape))
}

/// Puts the text of each container-bound text element into its container shape.
pub fn attach_text(live: &[&Element], imported: &mut Imported, ids: &mut Ids) {
    for element in live.iter().filter(|e| e.kind == "text") {
        let Some(container) = element.container_id.as_deref() else { continue };
        let id = ids.get(container);
        let Some(shape) = imported.shapes.iter_mut().find(|s| s.id == id && s.kind.has_text()) else { continue };
        shape.text = element.text.clone().unwrap_or_default().chars().take(MAX_TEXT).collect();
        let text = style(element);
        (shape.style.size, shape.style.font, shape.style.align) = (text.size, text.font, text.align);
        shape.style.stroke = text.stroke.or(shape.style.stroke.take());
    }
}
