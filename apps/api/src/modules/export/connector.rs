use std::collections::HashMap;

use super::geometry;
use super::markup::{INK, color, escape, num};
use super::model::{Kind, Shape};

fn head(tip: [f64; 2], from: [f64; 2], width: f64) -> String {
    let angle = (tip[1] - from[1]).atan2(tip[0] - from[0]);
    let size = width * 3.0 + 6.0;
    let wing = |turn: f64| [tip[0] - size * (angle + turn).cos(), tip[1] - size * (angle + turn).sin()];
    let ([lx, ly], [rx, ry]) = (wing(0.45), wing(-0.45));
    format!("M{} {}L{} {}L{} {}Z", num(tip[0]), num(tip[1]), num(lx), num(ly), num(rx), num(ry))
}

/// A line or an arrow, bindings resolved, with its heads.
pub fn draw(shape: &Shape, by_id: &HashMap<&str, &Shape>) -> String {
    let points = geometry::path(shape, by_id);
    let stroke = escape(color(shape.style.stroke.as_deref(), INK));
    let width = shape.style.width.unwrap_or(2.0).clamp(0.5, 200.0);
    let list: Vec<String> = points.iter().map(|[x, y]| format!("{},{}", num(*x), num(*y))).collect();
    let mut out = format!(
        "<polyline points=\"{}\" fill=\"none\" stroke=\"{stroke}\" stroke-width=\"{}\" stroke-linecap=\"round\" \
         stroke-linejoin=\"round\"/>",
        list.join(" "),
        num(width)
    );
    let default = if shape.kind == Kind::Arrow { "end" } else { "none" };
    let heads = shape.head.as_deref().unwrap_or(default);
    let n = points.len();
    let mut tips = Vec::new();
    if matches!(heads, "end" | "both") {
        tips.push(head(points[n - 1], points[n - 2], width));
    }
    if matches!(heads, "start" | "both") {
        tips.push(head(points[0], points[1], width));
    }
    for tip in tips {
        out.push_str(&format!("<path d=\"{tip}\" fill=\"{stroke}\" stroke=\"{stroke}\" stroke-linejoin=\"round\"/>"));
    }
    out
}
