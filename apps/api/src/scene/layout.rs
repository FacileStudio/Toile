use super::read::Shape;

/// The gap between shapes the layout places, in world units.
pub const GAP: f64 = 40.0;
/// How far below a frame's top edge its first row of children starts.
pub const FRAME_TOP: f64 = 80.0;
/// How wide a row of the layout grows before it wraps.
pub const ROW: f64 = 1000.0;

const DIGITS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

/// A fractional index after `a`, the client's `keyBetween(a, null)`.
pub fn after(a: &str) -> String {
    let Some(first) = a.bytes().next() else { return "V".to_owned() };
    let d = DIGITS.iter().position(|c| *c == first).unwrap_or(0);
    match d < DIGITS.len() - 1 {
        true => (DIGITS[d + 1] as char).to_string(),
        false => format!("{}{}", first as char, after(&a[1..])),
    }
}

/// The size a shape of `kind` gets when the caller gives none: the client's click sizes.
pub fn default_size(kind: &str) -> (f64, f64) {
    match kind {
        "sticky" => (200.0, 200.0),
        "text" => (240.0, 32.0),
        "rect" => (180.0, 120.0),
        "ellipse" => (160.0, 160.0),
        "diamond" => (180.0, 140.0),
        "frame" => (800.0, 520.0),
        _ => (160.0, 120.0),
    }
}

/// The box around every shape that has one (arrows, lines and ink are left out):
/// `(left, top, right, bottom)`.
pub fn bounds<'a>(shapes: impl IntoIterator<Item = &'a Shape>) -> Option<(f64, f64, f64, f64)> {
    shapes.into_iter().filter(|s| !matches!(s.kind.as_str(), "arrow" | "line" | "ink")).fold(None, |acc, s| {
        let (l, t, r, b) = acc.unwrap_or((s.x, s.y, s.x + s.w, s.y + s.h));
        Some((l.min(s.x), t.min(s.y), r.max(s.x + s.w), b.max(s.y + s.h)))
    })
}

/// Rows of shapes, left to right, wrapping at a width: where the layout puts a shape the caller
/// did not position.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shelf {
    left: f64,
    width: f64,
    x: f64,
    y: f64,
    row: f64,
}

impl Shelf {
    /// A shelf whose first row starts at `(left, top)` and wraps past `width`.
    pub fn new(left: f64, top: f64, width: f64) -> Shelf {
        Shelf { left, width: width.max(1.0), x: left, y: top, row: 0.0 }
    }

    /// The top-left corner of the next `w` by `h` box.
    pub fn place(&mut self, w: f64, h: f64) -> (f64, f64) {
        if self.x > self.left && self.x + w > self.left + self.width {
            self.x = self.left;
            self.y += self.row + GAP;
            self.row = 0.0;
        }
        let at = (self.x, self.y);
        self.x += w + GAP;
        self.row = self.row.max(h);
        at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indexes_grow_like_the_client_and_shelves_wrap() {
        assert_eq!(
            (after(""), after("V"), after("z"), after("zz")),
            ("V".into(), "W".into(), "zV".into(), "zzV".into())
        );
        let mut shelf = Shelf::new(0.0, 0.0, 500.0);
        assert_eq!((shelf.place(200.0, 200.0), shelf.place(200.0, 100.0)), ((0.0, 0.0), (240.0, 0.0)));
        assert_eq!(shelf.place(200.0, 50.0), (0.0, 240.0));
    }
}
