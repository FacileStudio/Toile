use freedraw::{InputPoint, StrokeOptions, get_stroke, get_svg_path_from_stroke};

/// The perfect-freehand settings the canvas client draws ink with. Keep them in step with
/// `apps/client/src/lib/canvas`.
pub fn options(width: f64) -> StrokeOptions {
    StrokeOptions {
        size: Some(width.max(0.5) * 2.0),
        thinning: Some(0.5),
        smoothing: Some(0.5),
        streamline: Some(0.5),
        simulate_pressure: Some(false),
        last: Some(true),
        ..StrokeOptions::default()
    }
}

/// The filled outline of an ink stroke as SVG path data, in the stroke's own coordinates.
/// `points` is the stored flat `[x, y, pressure, ...]` list.
pub fn outline(points: &[f64], width: f64) -> String {
    let input: Vec<InputPoint> = points
        .as_chunks::<3>()
        .0
        .iter()
        .map(|[x, y, p]| InputPoint::Array([*x, *y], Some(p.clamp(0.0, 1.0))))
        .collect();
    if input.is_empty() {
        return String::new();
    }
    let stroke = get_stroke(&input, &options(width));
    if stroke.len() < 4 {
        return dot(points[0], points[1], width);
    }
    get_svg_path_from_stroke(&stroke, true)
}

fn dot(x: f64, y: f64, width: f64) -> String {
    let r = width.max(0.5);
    format!("M{},{} a{r},{r} 0 1,0 {},0 a{r},{r} 0 1,0 -{},0Z", x - r, y, r * 2.0, r * 2.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_stroke_becomes_a_closed_filled_outline() {
        let path = outline(&[0.0, 0.0, 0.5, 10.0, 5.0, 0.6, 20.0, 0.0, 0.7, 30.0, 10.0, 0.5], 4.0);
        assert!(path.starts_with('M') && path.ends_with('Z'), "{path}");
    }

    #[test]
    fn one_point_is_a_dot_and_none_is_nothing() {
        assert!(outline(&[5.0, 5.0, 0.5], 4.0).ends_with('Z'));
        assert!(dot(5.0, 5.0, 4.0).starts_with("M1,5"));
        assert_eq!(outline(&[], 4.0), "");
    }
}
