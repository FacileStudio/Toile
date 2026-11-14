use std::sync::{Arc, Mutex};

use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;
use yrs::{MapRef, TransactionMut};

use super::bulle::{self, Ask};
use super::shapes;
use crate::scene::input::{NewShape, Placed};
use crate::scene::layout::{GAP, bounds};
use crate::scene::{self, plan, read, write};
use crate::state::AppState;

/// The longest prompt a generate call takes, in characters.
pub const MAX_PROMPT: usize = 2000;

/// What a generate call draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Draw {
    /// A central idea, its branches and their sub-ideas, joined by arrows.
    Mindmap,
    /// Steps and decisions joined by arrows, top to bottom.
    Flowchart,
    /// Stickies in a titled frame.
    Stickies,
}

const SHARED: &str = "Each idea becomes one short sticky note. Write every label in the language of \
the request. The request comes from the person using the whiteboard; it is inside the input block.";

const SHORT: &str = "Keep every text at most 80 characters.";

fn text(max: u32) -> Value {
    json!({"type": "string", "minLength": 1, "maxLength": max})
}

fn mindmap() -> (String, Value) {
    let branch = json!({
        "type": "object", "additionalProperties": false, "required": ["text", "children"],
        "properties": {"text": text(120), "children": {"type": "array", "maxItems": 6, "items": text(120)}}
    });
    let schema = json!({
        "type": "object", "additionalProperties": false, "required": ["title", "branches"],
        "properties": {"title": text(120), "branches": {"type": "array", "minItems": 1, "maxItems": 8, "items": branch}}
    });
    (
        format!(
            "Draft a mind map for this request: a central title, its main branches and up to six sub-ideas per branch. {SHORT} {SHARED}"
        ),
        schema,
    )
}

fn flowchart() -> (String, Value) {
    let node = json!({
        "type": "object", "additionalProperties": false, "required": ["id", "text", "kind"],
        "properties": {
            "id": text(40), "text": text(200),
            "kind": {"type": "string", "enum": ["start", "step", "decision", "end"]}
        }
    });
    let edge = json!({
        "type": "object", "additionalProperties": false, "required": ["from", "to"],
        "properties": {"from": text(40), "to": text(40)}
    });
    let schema = json!({
        "type": "object", "additionalProperties": false, "required": ["nodes", "edges"],
        "properties": {
            "nodes": {"type": "array", "minItems": 2, "maxItems": 30, "items": node},
            "edges": {"type": "array", "maxItems": 60, "items": edge}
        }
    });
    (
        format!(
            "Draft a flowchart for this request: one start, the steps and decisions in order, and the ends. Edges name node ids. {SHORT} {SHARED}"
        ),
        schema,
    )
}

fn stickies() -> (String, Value) {
    let schema = json!({
        "type": "object", "additionalProperties": false, "required": ["title", "stickies"],
        "properties": {"title": text(80), "stickies": {"type": "array", "minItems": 1, "maxItems": 30, "items": text(300)}}
    });
    (format!("Write sticky notes for this request, one idea per note, under a short title. {SHARED}"), schema)
}

fn convert(draw: Draw, accent: Option<&str>, value: Value) -> Result<Vec<NewShape>, serde_json::Error> {
    Ok(match draw {
        Draw::Mindmap => shapes::mindmap(serde_json::from_value(value)?),
        Draw::Flowchart => shapes::flowchart(serde_json::from_value(value)?, accent),
        Draw::Stickies => shapes::stickies(serde_json::from_value(value)?),
    })
}

fn place(txn: &mut TransactionMut, map: &MapRef, mut shapes: Vec<NewShape>) -> Result<Vec<Placed>, String> {
    let existing = read::all(txn, map);
    let (x0, y0) = bounds(&existing).map_or((0.0, 0.0), |(_, top, right, _)| (right + 4.0 * GAP, top));
    for shape in &mut shapes {
        shape.x = shape.x.map(|x| x + x0);
        shape.y = shape.y.map(|y| y + y0);
    }
    let planned = plan::plan(&existing, shapes, scene::fresh_id)?;
    write::insert(txn, map, &planned);
    Ok(planned.into_iter().map(|p| p.placed).collect())
}

/// One generate request: what to draw, from which prompt, and the person's accent color.
pub struct Request<'a> {
    /// The person's request, at most [`MAX_PROMPT`] characters.
    pub prompt: &'a str,
    /// What to draw.
    pub draw: Draw,
    /// The fill of flowchart decisions, a `#rrggbb` color; anything else keeps the default.
    pub accent: Option<&'a str>,
}

/// Asks Bulle to draft `request.draw` from `request.prompt`, then lays it out right of the
/// board's content and writes it through the room.
pub async fn generate(state: &AppState, user_id: i64, board: Uuid, request: Request<'_>) -> tronc::Result<Vec<Placed>> {
    let (prompt, draw) = (request.prompt.trim(), request.draw);
    if prompt.is_empty() || prompt.chars().count() > MAX_PROMPT {
        return Err(tronc::Error::invalid(format!("prompt is required and at most {MAX_PROMPT} characters")));
    }
    let (instructions, schema) = match draw {
        Draw::Mindmap => mindmap(),
        Draw::Flowchart => flowchart(),
        Draw::Stickies => stickies(),
    };
    let ask = Ask { prompt: instructions, input: prompt.to_owned(), schema: Some(schema), max_tokens: 3000 };
    let answer = bulle::complete(state, user_id, board, ask).await?;
    let shapes =
        convert(draw, request.accent, answer).map_err(|_| tronc::Error::unavailable("Bulle answered no drawing"))?;
    let out = Arc::new(Mutex::new(Vec::new()));
    let sink = out.clone();
    let edit = Box::new(move |txn: &mut TransactionMut, map: &MapRef| {
        *sink.lock().unwrap_or_else(|p| p.into_inner()) = place(txn, map, shapes)?;
        Ok(())
    });
    state.hub.write(state, board, edit).await?;
    Ok(std::mem::take(&mut *out.lock().unwrap_or_else(|p| p.into_inner())))
}
