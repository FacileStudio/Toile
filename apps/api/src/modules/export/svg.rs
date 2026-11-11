use std::fmt::Write;

use super::geometry::FRAME_LABEL;
use super::markup::{SANS, color, escape, num, transform};
use super::model::{Kind, Shape};
use super::scene::Scene;
use super::shapes::{self, MUTED};

const FRAME_FILL: &str = "#fafafa";

/// What to draw and how.
#[derive(Debug, Clone, Copy, Default)]
pub struct Options<'a> {
    /// Only this frame and what it holds.
    pub frame: Option<&'a str>,
    /// A colour painted under everything; `None` leaves the SVG transparent.
    pub background: Option<&'a str>,
    /// The URL prefix an image's `asset` id is appended to.
    pub asset_base: &'a str,
}

/// A board drawn as one standalone SVG document, with its size in board units.
#[derive(Debug, Clone)]
pub struct Drawing {
    pub svg: String,
    pub width: f64,
    pub height: f64,
}

struct Writer<'s, 'a> {
    scene: &'s Scene<'a>,
    options: &'s Options<'s>,
    out: String,
    clips: usize,
}

impl Writer<'_, '_> {
    fn node(&mut self, shape: &Shape) {
        match shape.kind {
            Kind::Frame => self.frame(shape),
            _ => self.out.push_str(&shapes::draw(shape, &self.scene.by_id, self.options.asset_base)),
        }
    }

    fn frame(&mut self, shape: &Shape) {
        let (w, h, place) = (num(shape.w), num(shape.h), transform(shape));
        let fill = escape(color(shape.style.fill.as_deref(), FRAME_FILL));
        let name = escape(&shape.name);
        let label = num(-FRAME_LABEL / 2.0);
        let _ = write!(
            self.out,
            "<g transform=\"{place}\"><rect width=\"{w}\" height=\"{h}\" fill=\"{fill}\"/><text y=\"{label}\" \
             font-family=\"{}\" font-size=\"14\" fill=\"{MUTED}\">{name}</text></g>",
            escape(SANS)
        );
        let children = self.scene.children(&shape.id);
        if children.is_empty() {
            return;
        }
        self.clips += 1;
        let clip = self.clips;
        let _ = write!(
            self.out,
            "<clipPath id=\"frame-{clip}\"><rect width=\"{w}\" height=\"{h}\" transform=\"{place}\"/></clipPath>\
             <g clip-path=\"url(#frame-{clip})\">"
        );
        for child in children {
            self.node(child);
        }
        self.out.push_str("</g>");
    }
}

/// Draws `shapes` as SVG, in z order, frames clipping what they hold. `None` when
/// `options.frame` names no frame of the board.
pub fn render(shapes: &[Shape], options: &Options) -> Option<Drawing> {
    let scene = Scene::new(shapes, options.frame)?;
    let b = scene.bounds;
    let (width, height) = (b.x1 - b.x0, b.y1 - b.y0);
    let mut writer = Writer { scene: &scene, options, out: String::new(), clips: 0 };
    let _ = write!(
        writer.out,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"{} {} {} {}\">",
        num(width),
        num(height),
        num(b.x0),
        num(b.y0),
        num(width),
        num(height)
    );
    if let Some(background) = options.background {
        let fill = escape(color(Some(background), "#ffffff"));
        let _ = write!(
            writer.out,
            "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"{fill}\"/>",
            num(b.x0),
            num(b.y0),
            num(width),
            num(height)
        );
    }
    for shape in &scene.roots {
        writer.node(shape);
    }
    writer.out.push_str("</svg>");
    Some(Drawing { svg: writer.out, width, height })
}
