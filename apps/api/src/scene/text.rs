use std::collections::HashSet;
use std::fmt::Write;

use super::read::Shape;

fn quoted(text: &str) -> String {
    serde_json::to_string(text).unwrap_or_default()
}

fn end(id: &Option<String>) -> &str {
    id.as_deref().unwrap_or("a point")
}

fn color(shape: &Shape) -> String {
    match shape.color.is_empty() {
        true => String::new(),
        false => format!(" {}", shape.color),
    }
}

/// One line for `shape`: type, id, where it sits, and what it says.
pub fn line(shape: &Shape) -> String {
    let (x, y, w, h) = (shape.x.round(), shape.y.round(), shape.w.round(), shape.h.round());
    let at = format!("{} {} at {x},{y} {w}x{h}{}", shape.kind, shape.id, color(shape));
    match shape.kind.as_str() {
        "arrow" | "line" => format!("{} {} from {} to {}", shape.kind, shape.id, end(&shape.from), end(&shape.to)),
        "card" => format!("{at}: {} <{}>", quoted(&shape.title), shape.url),
        "frame" => format!("frame {} {} at {x},{y} {w}x{h}", shape.id, quoted(&shape.name)),
        "ink" | "image" => at,
        _ => format!("{at}: {}", quoted(&shape.text)),
    }
}

/// `shapes` as text an agent reads: each frame then its children indented, then every shape
/// outside a frame.
pub fn render(shapes: &[Shape]) -> String {
    let frames: HashSet<&str> = shapes.iter().filter(|s| s.kind == "frame").map(|s| s.id.as_str()).collect();
    let mut out = String::new();
    for frame in shapes.iter().filter(|s| s.kind == "frame") {
        _ = writeln!(out, "{}", line(frame));
        for child in shapes.iter().filter(|s| s.parent.as_deref() == Some(frame.id.as_str())) {
            _ = writeln!(out, "  {}", line(child));
        }
    }
    let loose = |s: &&Shape| s.kind != "frame" && s.parent.as_deref().is_none_or(|p| !frames.contains(p));
    let outside: Vec<&Shape> = shapes.iter().filter(loose).collect();
    if !outside.is_empty() && !frames.is_empty() {
        out.push_str("outside frames:\n");
    }
    for shape in outside {
        _ = writeln!(out, "{}", line(shape));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shape(id: &str, kind: &str, parent: Option<&str>) -> Shape {
        let parent = parent.map(str::to_owned);
        Shape { id: id.into(), kind: kind.into(), w: 200.0, h: 200.0, parent, ..Shape::default() }
    }

    #[test]
    fn frames_group_their_children_and_arrows_name_their_ends() {
        let mut frame = shape("f", "frame", None);
        frame.name = "Ideas".into();
        let mut sticky = shape("s", "sticky", Some("f"));
        sticky.text = "Ship \"it\"".into();
        let mut arrow = shape("a", "arrow", None);
        arrow.from = Some("s".into());
        let text = render(&[frame, sticky, arrow]);
        let expected = "frame f \"Ideas\" at 0,0 200x200\n  sticky s at 0,0 200x200: \"Ship \\\"it\\\"\"\n\
                        outside frames:\narrow a from s to a point\n";
        assert_eq!(text, expected);
    }
}
