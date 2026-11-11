use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The shape types an agent or the AI may create.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Sticky,
    Text,
    Rect,
    Ellipse,
    Diamond,
    Frame,
    Arrow,
    Line,
}

impl Kind {
    /// The schema 1 `type`.
    pub fn name(self) -> &'static str {
        match self {
            Kind::Sticky => "sticky",
            Kind::Text => "text",
            Kind::Rect => "rect",
            Kind::Ellipse => "ellipse",
            Kind::Diamond => "diamond",
            Kind::Frame => "frame",
            Kind::Arrow => "arrow",
            Kind::Line => "line",
        }
    }

    /// Whether the shape is drawn from two ends rather than a box.
    pub fn is_path(self) -> bool {
        matches!(self, Kind::Arrow | Kind::Line)
    }

    /// Whether the shape's color is its stroke rather than its fill.
    pub fn stroked(self) -> bool {
        matches!(self, Kind::Text | Kind::Arrow | Kind::Line)
    }
}

/// A shape to add. Leave out `x` and `y` and Toile places it: inside its frame, or to the right
/// of the board's content.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct NewShape {
    /// A name for this shape within this call only, so `from`, `to` and `frame` of other new
    /// shapes can point at it.
    #[serde(rename = "ref", default, skip_serializing_if = "Option::is_none")]
    pub handle: Option<String>,
    /// sticky, text, rect, ellipse, diamond, frame, arrow or line.
    #[serde(rename = "type")]
    pub kind: Kind,
    /// The label: a sticky's or box's text, a frame's title. Arrows and lines take none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub y: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub w: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub h: Option<f64>,
    /// A CSS color, such as `#fde68a`: the fill of stickies, boxes and frames, the stroke of
    /// text, arrows and lines.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// The frame this shape sits in: an existing frame id, or the `ref` of a new frame.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frame: Option<String>,
    /// Arrows and lines: the shape the arrow starts at (an id or a `ref`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    /// Arrows and lines: the shape the arrow points at (an id or a `ref`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
}

impl NewShape {
    /// A shape of `kind` with `text`, placed by the layout.
    pub fn new(kind: Kind, text: impl Into<String>) -> NewShape {
        let text = Some(text.into()).filter(|t| !t.is_empty());
        NewShape {
            handle: None,
            kind,
            text,
            x: None,
            y: None,
            w: None,
            h: None,
            color: None,
            frame: None,
            from: None,
            to: None,
        }
    }

    /// When it is planned: frames first, then boxes, then arrows and lines.
    pub fn rank(&self) -> u8 {
        match self.kind {
            Kind::Frame => 0,
            kind if kind.is_path() => 2,
            _ => 1,
        }
    }

    /// Whether the layout places it: a box without both `x` and `y`.
    pub fn floating(&self) -> bool {
        !self.kind.is_path() && (self.x.is_none() || self.y.is_none())
    }
}

/// Where a new shape landed, as the tools answer it.
#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct Placed {
    pub id: String,
    /// The `ref` the call gave it.
    #[serde(rename = "ref", skip_serializing_if = "Option::is_none")]
    pub handle: Option<String>,
    #[serde(rename = "type")]
    pub kind: Kind,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

/// A new shape, ready to write.
#[derive(Debug, Clone, PartialEq)]
pub struct Planned {
    pub placed: Placed,
    pub z: String,
    pub parent: Option<String>,
    pub text: String,
    pub color: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    /// Arrows and lines: the far end, relative to `x`, `y`.
    pub end: (f64, f64),
}

impl Planned {
    /// `placed` with its label, color and frame; no ends and no paint order yet.
    pub fn new(placed: Placed, text: &Option<String>, color: Option<String>, parent: Option<String>) -> Planned {
        let text = text.clone().unwrap_or_default();
        Planned { placed, z: String::new(), parent, text, color, from: None, to: None, end: (0.0, 0.0) }
    }
}
