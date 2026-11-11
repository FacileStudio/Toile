use std::collections::HashMap;

use super::connector;
use super::ink;
use super::markup::{self, INK, Placement, color, escape, num, transform};
use super::model::{Kind, Shape};

const STICKY: &str = "#fde68a";
const CARD: &str = "#f4f4f5";
/// The colour of secondary text.
pub const MUTED: &str = "#71717a";

fn place(shape: &Shape) -> String {
    format!("<g transform=\"{}\">", transform(shape))
}

fn paint(shape: &Shape, fill: &str) -> String {
    let outlined = matches!(shape.kind, Kind::Rect | Kind::Ellipse | Kind::Diamond);
    let fill = color(shape.style.fill.as_deref(), fill);
    let stroke = color(shape.style.stroke.as_deref(), if outlined { INK } else { "none" });
    let width = num(shape.style.width.unwrap_or(2.0).clamp(0.0, 200.0));
    format!("fill=\"{}\" stroke=\"{}\" stroke-width=\"{width}\"", escape(fill), escape(stroke))
}

fn geometry_of(shape: &Shape) -> String {
    let (w, h) = (num(shape.w), num(shape.h));
    match shape.kind {
        Kind::Ellipse => {
            let (rx, ry) = (num(shape.w / 2.0), num(shape.h / 2.0));
            format!("<ellipse cx=\"{rx}\" cy=\"{ry}\" rx=\"{rx}\" ry=\"{ry}\" {}/>", paint(shape, "none"))
        }
        Kind::Diamond => {
            let (mx, my) = (num(shape.w / 2.0), num(shape.h / 2.0));
            let points = format!("{mx},0 {w},{my} {mx},{h} 0,{my}");
            format!("<polygon points=\"{points}\" {}/>", paint(shape, "none"))
        }
        Kind::Sticky => format!("<rect width=\"{w}\" height=\"{h}\" rx=\"6\" {}/>", paint(shape, STICKY)),
        Kind::Text => String::new(),
        _ => format!("<rect width=\"{w}\" height=\"{h}\" rx=\"4\" {}/>", paint(shape, "none")),
    }
}

fn boxed(shape: &Shape) -> String {
    let middle = shape.kind != Kind::Text;
    let at = Placement { w: shape.w, h: shape.h, padding: if middle { 12.0 } else { 0.0 }, middle };
    let fill = color(shape.style.stroke.as_deref(), INK);
    let label = markup::block(&shape.text, &shape.style, at, fill);
    format!("{}{}{label}</g>", place(shape), geometry_of(shape))
}

fn stroke(shape: &Shape) -> String {
    let fill = escape(color(shape.style.stroke.as_deref(), INK));
    let path = ink::outline(&shape.points, shape.style.width.unwrap_or(2.0).clamp(0.5, 200.0));
    format!("{}<path d=\"{path}\" fill=\"{fill}\"/></g>", place(shape))
}

fn media(shape: &Shape, asset_base: &str) -> String {
    let (w, h) = (num(shape.w), num(shape.h));
    let id: String = shape.asset.chars().filter(|c| c.is_ascii_alphanumeric() || "-_".contains(*c)).collect();
    let href = escape(&format!("{asset_base}{id}"));
    format!(
        "{}<rect width=\"{w}\" height=\"{h}\" fill=\"{CARD}\"/><image href=\"{href}\" width=\"{w}\" height=\"{h}\" \
         preserveAspectRatio=\"xMidYMid meet\"/></g>",
        place(shape)
    )
}

fn card(shape: &Shape) -> String {
    let title = if shape.title.is_empty() { &shape.url } else { &shape.title };
    let mut style = shape.style.clone();
    style.align = Some("left".to_owned());
    let at = |h: f64| Placement { w: shape.w, h, padding: 14.0, middle: false };
    let heading = markup::block(title, &style, at(shape.h), INK);
    style.size = Some(style.size.unwrap_or(16.0) * 0.75);
    let link = format!(
        "<g transform=\"translate(0 {})\">{}</g>",
        num(shape.h - 34.0),
        markup::block(&shape.url, &style, at(34.0), MUTED)
    );
    format!(
        "{}<rect width=\"{}\" height=\"{}\" rx=\"10\" {}/>{heading}{link}</g>",
        place(shape),
        num(shape.w),
        num(shape.h),
        paint(shape, CARD)
    )
}

/// The SVG for one shape that is not a frame, in board coordinates.
pub fn draw(shape: &Shape, by_id: &HashMap<&str, &Shape>, asset_base: &str) -> String {
    match shape.kind {
        Kind::Line | Kind::Arrow => connector::draw(shape, by_id),
        Kind::Ink => stroke(shape),
        Kind::Image => media(shape, asset_base),
        Kind::Card => card(shape),
        Kind::Frame => String::new(),
        _ => boxed(shape),
    }
}
