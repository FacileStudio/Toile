use std::fmt::Write;

use super::model::{Shape, Style};

/// muse's body font stack (`--font-fc-body`). Nothing is embedded: the reader's fonts apply.
pub const SANS: &str =
    "ui-sans-serif, system-ui, -apple-system, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif";

/// muse's monospace stack (`--font-fc-mono`).
pub const MONO: &str = "ui-monospace, 'SF Mono', 'JetBrains Mono', Menlo, Consolas, monospace";

/// The ink colour shapes fall back to.
pub const INK: &str = "#1f1f23";

const LINE_HEIGHT: f64 = 1.3;
const GLYPH: f64 = 0.56;

/// `raw` safe inside an XML attribute or text node.
pub fn escape(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for c in raw.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c if c.is_control() && c != '\n' && c != '\t' => {}
            c => out.push(c),
        }
    }
    out
}

/// A stored colour when it is a plain CSS colour (hex, a name, or an `rgb()`/`hsl()` call),
/// else `fallback`. Anything else, `url(...)` included, never reaches the SVG.
pub fn color<'a>(raw: Option<&'a str>, fallback: &'a str) -> &'a str {
    let Some(raw) = raw.map(str::trim) else { return fallback };
    let hex = raw.len() <= 9 && raw.starts_with('#') && raw[1..].chars().all(|c| c.is_ascii_hexdigit());
    let name = !raw.is_empty() && raw.len() <= 32 && raw.chars().all(|c| c.is_ascii_alphabetic());
    let call = ["rgb(", "rgba(", "hsl(", "hsla("].iter().any(|f| raw.starts_with(f))
        && raw.ends_with(')')
        && raw.len() <= 48
        && raw.chars().all(|c| c.is_ascii_alphanumeric() || "(),.% ".contains(c));
    if hex || name || call { raw } else { fallback }
}

/// `text` broken into lines that fit `width` at `size`, estimating glyph widths.
pub fn wrap(text: &str, size: f64, width: f64) -> Vec<String> {
    let per_line = ((width / (size * GLYPH)).floor() as usize).max(1);
    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        let mut line = String::new();
        for word in paragraph.split(' ') {
            let fits = line.chars().count() + 1 + word.chars().count() <= per_line;
            if !line.is_empty() && !fits {
                lines.push(std::mem::take(&mut line));
            }
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(word);
        }
        lines.push(line);
    }
    lines
}

/// Where a block of text sits inside its box.
pub struct Placement {
    pub w: f64,
    pub h: f64,
    pub padding: f64,
    pub middle: bool,
}

/// A `<text>` element for `text` laid out in a `w` by `h` box at the local origin.
pub fn block(text: &str, style: &Style, at: Placement, fill: &str) -> String {
    if text.trim().is_empty() {
        return String::new();
    }
    let size = style.size.unwrap_or(16.0).clamp(4.0, 400.0);
    let family = if style.font.as_deref() == Some("mono") { MONO } else { SANS };
    let (anchor, x) = match style.align.as_deref() {
        Some("left") => ("start", at.padding),
        Some("right") => ("end", at.w - at.padding),
        _ => ("middle", at.w / 2.0),
    };
    let lines = wrap(text, size, (at.w - 2.0 * at.padding).max(size));
    let total = lines.len() as f64 * size * LINE_HEIGHT;
    let top = if at.middle { (at.h - total) / 2.0 } else { at.padding };
    let mut out = format!(
        "<text font-family=\"{}\" font-size=\"{}\" fill=\"{}\" text-anchor=\"{anchor}\">",
        escape(family),
        num(size),
        escape(fill)
    );
    for (i, line) in lines.iter().enumerate() {
        let y = top + size * (i as f64 * LINE_HEIGHT + 1.0);
        let _ = write!(out, "<tspan x=\"{}\" y=\"{}\">{}</tspan>", num(x), num(y), escape(line));
    }
    out.push_str("</text>");
    out
}

/// A number written for SVG: at most two decimals, no trailing zeros.
pub fn num(value: f64) -> String {
    let value = if value.is_finite() { (value * 100.0).round() / 100.0 } else { 0.0 };
    let text = format!("{value:.2}");
    let text = text.trim_end_matches('0').trim_end_matches('.');
    if text == "-0" { "0".to_owned() } else { text.to_owned() }
}

/// The transform that places a box shape: moved to `x`,`y`, turned around its centre.
pub fn transform(shape: &Shape) -> String {
    let mut transform = format!("translate({} {})", num(shape.x), num(shape.y));
    if shape.rot != 0.0 {
        let degrees = shape.rot.to_degrees();
        transform.push_str(&format!(" rotate({} {} {})", num(degrees), num(shape.w / 2.0), num(shape.h / 2.0)));
    }
    transform
}
