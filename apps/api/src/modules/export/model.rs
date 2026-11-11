/// The shape types of board schema 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Sticky,
    Text,
    Rect,
    Ellipse,
    Diamond,
    Line,
    Arrow,
    Ink,
    Image,
    Card,
    Frame,
}

const KINDS: [(&str, Kind); 11] = [
    ("sticky", Kind::Sticky),
    ("text", Kind::Text),
    ("rect", Kind::Rect),
    ("ellipse", Kind::Ellipse),
    ("diamond", Kind::Diamond),
    ("line", Kind::Line),
    ("arrow", Kind::Arrow),
    ("ink", Kind::Ink),
    ("image", Kind::Image),
    ("card", Kind::Card),
    ("frame", Kind::Frame),
];

impl Kind {
    /// The `type` value of a shape, or `None` for a type schema 1 does not know.
    pub fn parse(raw: &str) -> Option<Kind> {
        KINDS.iter().find(|(name, _)| *name == raw).map(|(_, kind)| *kind)
    }

    /// The `type` value stored in the document.
    pub fn name(self) -> &'static str {
        KINDS.iter().find(|(_, kind)| *kind == self).map_or("rect", |(name, _)| name)
    }

    /// Whether the shape carries a `text` Y.Text.
    pub fn has_text(self) -> bool {
        matches!(self, Kind::Sticky | Kind::Text | Kind::Rect | Kind::Ellipse | Kind::Diamond)
    }
}

/// The plain `style` object of a shape. Absent keys take the renderer's defaults.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Style {
    pub fill: Option<String>,
    pub stroke: Option<String>,
    pub width: Option<f64>,
    pub font: Option<String>,
    pub size: Option<f64>,
    pub align: Option<String>,
}

/// An arrow end bound to a shape: `anchor` is a fraction of the target's box.
#[derive(Debug, Clone, PartialEq)]
pub struct Binding {
    pub shape: String,
    pub anchor: [f64; 2],
}

/// One shape of a board, flattened from its Y.Map. Keys a type does not use stay empty.
#[derive(Debug, Clone, PartialEq)]
pub struct Shape {
    pub id: String,
    pub kind: Kind,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub rot: f64,
    pub z: String,
    pub parent: Option<String>,
    pub style: Style,
    pub text: String,
    pub points: Vec<f64>,
    pub from: Option<Binding>,
    pub to: Option<Binding>,
    pub head: Option<String>,
    pub asset: String,
    pub url: String,
    pub title: String,
    pub card_kind: String,
    pub name: String,
}

impl Shape {
    /// An empty shape of `kind` at the origin, for importers to fill.
    pub fn new(id: impl Into<String>, kind: Kind) -> Shape {
        Shape {
            id: id.into(),
            kind,
            x: 0.0,
            y: 0.0,
            w: 0.0,
            h: 0.0,
            rot: 0.0,
            z: String::new(),
            parent: None,
            style: Style::default(),
            text: String::new(),
            points: Vec::new(),
            from: None,
            to: None,
            head: None,
            asset: String::new(),
            url: String::new(),
            title: String::new(),
            card_kind: String::new(),
            name: String::new(),
        }
    }
}
