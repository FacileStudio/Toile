use std::sync::{Arc, OnceLock};

use resvg::{tiny_skia, usvg};
use tronc::{Error, Result};

use super::svg::Drawing;

static DEJAVU_SANS: &[u8] = include_bytes!("../../../fonts/DejaVuSans.ttf");

/// The longest side, in pixels, a PNG export may have.
pub const MAX_SIDE: u32 = 4096;

fn options() -> &'static usvg::Options<'static> {
    static OPTIONS: OnceLock<usvg::Options<'static>> = OnceLock::new();
    OPTIONS.get_or_init(|| {
        let mut fonts = usvg::fontdb::Database::new();
        fonts.load_font_data(DEJAVU_SANS.to_vec());
        fonts.set_sans_serif_family("DejaVu Sans");
        fonts.set_monospace_family("DejaVu Sans");
        let resolver = usvg::ImageHrefResolver {
            resolve_data: usvg::ImageHrefResolver::default_data_resolver(),
            resolve_string: Box::new(|_, _| None),
        };
        usvg::Options {
            font_family: "DejaVu Sans".to_owned(),
            fontdb: Arc::new(fonts),
            image_href_resolver: resolver,
            ..usvg::Options::default()
        }
    })
}

fn tree(drawing: &Drawing) -> Result<usvg::Tree> {
    usvg::Tree::from_str(&drawing.svg, options()).map_err(|e| Error::internal("svg parse failed", e))
}

/// The pixel size of a PNG of `drawing` at `scale`, shrunk so no side exceeds `max_side`.
pub fn size(drawing: &Drawing, scale: f64, max_side: u32) -> (u32, u32) {
    let longest = drawing.width.max(drawing.height).max(1.0);
    let scale = scale.min(f64::from(max_side) / longest);
    let px = |v: f64| ((v * scale).round() as u32).clamp(1, max_side);
    (px(drawing.width), px(drawing.height))
}

/// `drawing` rasterized to PNG at `scale`, no side over `max_side` pixels. Images the SVG
/// links to are not fetched: only the vector content is drawn.
pub fn png(drawing: &Drawing, scale: f64, max_side: u32) -> Result<Vec<u8>> {
    let tree = tree(drawing)?;
    let (w, h) = size(drawing, scale, max_side);
    let mut pixmap = tiny_skia::Pixmap::new(w, h).ok_or_else(|| Error::invalid("the drawing is empty"))?;
    let sx = w as f32 / tree.size().width();
    let sy = h as f32 / tree.size().height();
    resvg::render(&tree, tiny_skia::Transform::from_scale(sx, sy), &mut pixmap.as_mut());
    pixmap.encode_png().map_err(|e| Error::internal("png encoding failed", e))
}

/// The longest side, in points, of a PDF page; larger boards are scaled down to it.
pub const MAX_PAGE: f64 = 14_400.0;

/// `drawing` as a one-page PDF, one board unit to one point up to [`MAX_PAGE`], text in
/// embedded DejaVu Sans.
pub fn pdf(drawing: &Drawing) -> Result<Vec<u8>> {
    let tree = tree(drawing)?;
    let shrink = (drawing.width.max(drawing.height) / MAX_PAGE).max(1.0);
    let page = svg2pdf::PageOptions { dpi: (72.0 * shrink) as f32 };
    svg2pdf::to_pdf(&tree, svg2pdf::ConversionOptions::default(), page)
        .map_err(|e| Error::internal("pdf conversion failed", e.to_string()))
}
