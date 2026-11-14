use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;
use yrs::{Map, MapRef, Out, TransactionMut};

use super::bulle::{self, Ask};
use crate::scene::change::{self, Change};
use crate::scene::input::{Kind, NewShape};
use crate::scene::layout::{FRAME_TOP, GAP, ROW, Shelf, after, bounds};
use crate::scene::read::{self, Shape};
use crate::scene::{self, plan, write};
use crate::state::AppState;

/// The most shapes one cluster call sorts.
pub const MAX_SHAPES: usize = 200;
const MAX_GROUPS: usize = 12;

const PROMPT: &str = "Group these sticky notes by theme. Each note is one JSON object with its id \
and text. Put every note in exactly one group, copy each id exactly as given, and give each group \
a short title in the notes' language. Use 2 to 8 groups when there are enough notes.";

/// One theme Bulle found, and later the frame that holds it.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Group {
    /// The theme, the frame's title.
    pub title: String,
    /// The stickies in it.
    pub shapes: Vec<String>,
}

#[derive(Deserialize)]
struct Groups {
    groups: Vec<Group>,
}

/// A frame the cluster made, with the stickies moved into it.
#[derive(Debug, Clone, Serialize)]
pub struct Framed {
    /// The new frame's id.
    pub frame: String,
    /// Its title.
    pub title: String,
    /// The stickies now inside it.
    pub shapes: Vec<String>,
}

fn schema() -> Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["groups"],
        "properties": {"groups": {
            "type": "array", "minItems": 1, "maxItems": MAX_GROUPS,
            "items": {
                "type": "object",
                "additionalProperties": false,
                "required": ["title", "shapes"],
                "properties": {
                    "title": {"type": "string", "minLength": 1, "maxLength": 80},
                    "shapes": {"type": "array", "minItems": 1, "items": {"type": "string"}}
                }
            }
        }}
    })
}

fn stickies(shapes: &[Shape], ids: &[String]) -> Vec<Shape> {
    let wanted: HashSet<&str> = ids.iter().map(String::as_str).collect();
    let sticky = |s: &&Shape| s.kind == "sticky" && wanted.contains(s.id.as_str()) && !s.text.trim().is_empty();
    shapes.iter().filter(sticky).cloned().collect()
}

/// Keeps the groups' ids that are among `known`, each once, and drops empty groups.
pub fn clean(groups: Vec<Group>, known: &[Shape]) -> Vec<Group> {
    let mut seen: HashSet<String> = HashSet::new();
    let known: HashSet<&str> = known.iter().map(|s| s.id.as_str()).collect();
    let mut out = Vec::new();
    for group in groups.into_iter().take(MAX_GROUPS) {
        let shapes: Vec<String> =
            group.shapes.into_iter().filter(|id| known.contains(id.as_str()) && seen.insert(id.clone())).collect();
        let title: String = group.title.trim().chars().take(80).collect();
        if !shapes.is_empty() {
            out.push(Group { title, shapes });
        }
    }
    out
}

fn cell(members: &[&Shape]) -> (f64, f64, usize) {
    let w = members.iter().map(|s| s.w).fold(0.0, f64::max).max(1.0);
    let h = members.iter().map(|s| s.h).fold(0.0, f64::max).max(1.0);
    (w, h, (members.len() as f64).sqrt().ceil().max(1.0) as usize)
}

/// The frames and sticky moves for `groups`, laid out in rows from the stickies' top-left.
pub fn layout(groups: &[Group], live: &[Shape]) -> (Vec<NewShape>, Vec<Change>) {
    let members: Vec<Vec<&Shape>> =
        groups.iter().map(|g| g.shapes.iter().filter_map(|id| live.iter().find(|s| &s.id == id)).collect()).collect();
    let all = members.iter().flatten().copied();
    let (left, top, right, _) = bounds(all).unwrap_or((0.0, 0.0, 0.0, 0.0));
    let mut shelf = Shelf::new(left, top, (right - left).max(ROW));
    let (mut frames, mut moves) = (Vec::new(), Vec::new());
    for (i, (group, members)) in groups.iter().zip(&members).enumerate() {
        let (cw, ch, cols) = cell(members);
        let rows = members.len().div_ceil(cols);
        let (w, h) = (GAP + cols as f64 * (cw + GAP), FRAME_TOP + rows as f64 * (ch + GAP));
        let (x, y) = shelf.place(w, h);
        let handle = format!("group-{i}");
        let mut frame = NewShape::new(Kind::Frame, group.title.clone());
        (frame.handle, frame.x, frame.y, frame.w, frame.h) = (Some(handle.clone()), Some(x), Some(y), Some(w), Some(h));
        frames.push(frame);
        for (n, sticky) in members.iter().enumerate() {
            let (col, row) = ((n % cols) as f64, (n / cols) as f64);
            let at = (x + GAP + col * (cw + GAP), y + FRAME_TOP + row * (ch + GAP));
            moves.push(moved(&sticky.id, at, handle.clone()));
        }
    }
    (frames, moves)
}

fn moved(id: &str, (x, y): (f64, f64), frame: String) -> Change {
    Change { id: id.to_owned(), text: None, x: Some(x), y: Some(y), w: None, h: None, color: None, frame: Some(frame) }
}

fn raise(txn: &mut TransactionMut, shapes: &MapRef, ids: &[String], mut z: String) {
    for id in ids {
        z = after(&z);
        if let Some(Out::YMap(map)) = shapes.get(txn, id) {
            map.insert(txn, "z", z.as_str());
        }
    }
}

fn apply(txn: &mut TransactionMut, shapes: &MapRef, groups: &[Group]) -> Result<Vec<Framed>, String> {
    let live = read::all(txn, shapes);
    let groups = clean(groups.to_vec(), &live.iter().filter(|s| s.kind == "sticky").cloned().collect::<Vec<_>>());
    let (frames, mut moves) = layout(&groups, &live);
    let planned = plan::plan(&live, frames, scene::fresh_id)?;
    write::insert(txn, shapes, &planned);
    for change in &mut moves {
        let handle = change.frame.take().unwrap_or_default();
        change.frame =
            planned.iter().find(|p| p.placed.handle.as_deref() == Some(handle.as_str())).map(|p| p.placed.id.clone());
    }
    change::apply(txn, shapes, &moves)?;
    let top = planned.iter().map(|p| p.z.clone()).max().unwrap_or_default();
    raise(txn, shapes, &moves.iter().map(|m| m.id.clone()).collect::<Vec<_>>(), top);
    Ok(planned
        .into_iter()
        .zip(groups)
        .map(|(p, g)| Framed { frame: p.placed.id, title: g.title, shapes: g.shapes })
        .collect())
}

/// Asks Bulle to sort the selected stickies of `board` by theme, then frames each theme and
/// moves its stickies into the frame, in one room write.
pub async fn cluster(state: &AppState, user_id: i64, board: Uuid, ids: &[String]) -> tronc::Result<Vec<Framed>> {
    if ids.len() > MAX_SHAPES {
        return Err(tronc::Error::invalid(format!("cluster at most {MAX_SHAPES} shapes at once")));
    }
    let notes = stickies(&scene::load(state, board).await?, ids);
    if notes.len() < 2 {
        return Err(tronc::Error::invalid("select at least two stickies with text"));
    }
    let input: Vec<Value> =
        notes.iter().map(|s| json!({"id": s.id, "text": tronc::mcp::clip(&s.text, 500).0})).collect();
    let input = input.iter().map(Value::to_string).collect::<Vec<_>>().join("\n");
    let ask = Ask { prompt: PROMPT.to_owned(), input, schema: Some(schema()), max_tokens: 4096 };
    let answer: Groups = serde_json::from_value(bulle::complete(state, user_id, board, ask).await?)
        .map_err(|_| tronc::Error::unavailable("Bulle answered no groups"))?;
    let groups = clean(answer.groups, &notes);
    if groups.is_empty() {
        return Err(tronc::Error::unavailable("Bulle found no group for these stickies"));
    }
    let out = Arc::new(Mutex::new(Vec::new()));
    let sink = out.clone();
    let edit = Box::new(move |txn: &mut TransactionMut, shapes: &MapRef| {
        *sink.lock().unwrap_or_else(|p| p.into_inner()) = apply(txn, shapes, &groups)?;
        Ok(())
    });
    state.hub.write(state, board, edit).await?;
    Ok(std::mem::take(&mut *out.lock().unwrap_or_else(|p| p.into_inner())))
}
