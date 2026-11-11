use tronc::{Error, Result};
use uuid::Uuid;
use yrs::{Any, Map, MapPrelim, MapRef, Out, ReadTxn, Transact};

use crate::modules::board::{self, SHAPES};
use crate::modules::rooms::Edit;
use crate::state::AppState;

/// The parts of a sticky a task is made from.
#[derive(Debug, Clone, PartialEq)]
pub struct Sticky {
    /// Its text.
    pub text: String,
    /// Where the card goes: right of the sticky, top-aligned.
    pub x: f64,
    /// The sticky's top.
    pub y: f64,
    /// The sticky's z, when it has one.
    pub z: Option<String>,
}

fn number<T: ReadTxn>(txn: &T, shape: &MapRef, key: &str) -> Option<f64> {
    match shape.get(txn, key)? {
        Out::Any(Any::Number(value)) => value.as_f64(),
        _ => None,
    }
}

fn string<T: ReadTxn>(txn: &T, shape: &MapRef, key: &str) -> Option<String> {
    match shape.get(txn, key)? {
        Out::Any(Any::String(value)) => Some(value.to_string()),
        Out::YText(text) => Some(yrs::GetString::get_string(&text, txn)),
        _ => None,
    }
}

/// Reads sticky `id` from the live board: `404` when there is no such shape, `400` when it is
/// not a sticky or holds no text.
pub async fn read(state: &AppState, board_id: Uuid, id: &str) -> Result<Sticky> {
    let bytes = state.hub.read(state, board_id).await?;
    let doc = board::from_state(&bytes, []).map_err(|error| Error::internal("board decode", error))?;
    let shapes = doc.get_or_insert_map(SHAPES);
    let txn = doc.transact();
    let Some(Out::YMap(shape)) = shapes.get(&txn, id) else { return Err(Error::not_found("shape not found")) };
    if string(&txn, &shape, "type").as_deref() != Some("sticky") {
        return Err(Error::invalid("only a sticky becomes a task"));
    }
    let text = string(&txn, &shape, "text").unwrap_or_default().trim().to_owned();
    if text.is_empty() {
        return Err(Error::invalid("the sticky is empty"));
    }
    let (x, y) = (number(&txn, &shape, "x").unwrap_or(0.0), number(&txn, &shape, "y").unwrap_or(0.0));
    let w = number(&txn, &shape, "w").unwrap_or(200.0);
    Ok(Sticky { text, x: (x + w + 24.0).clamp(-1e7, 1e7), y, z: string(&txn, &shape, "z") })
}

/// A new shape id in the client's form: 16 URL-safe characters.
pub fn new_id() -> String {
    Uuid::new_v4().simple().to_string()[..16].to_owned()
}

/// The edit that adds a `card` shape `id` for the task at `url`, beside `sticky`, just above it.
pub fn card(id: String, sticky: &Sticky, url: String, title: String) -> Edit {
    let mut fields: Vec<(&str, Any)> = vec![
        ("type", "card".into()),
        ("x", sticky.x.into()),
        ("y", sticky.y.into()),
        ("w", 320.0.into()),
        ("h", 80.0.into()),
        ("url", url.into()),
        ("title", title.into()),
        ("kind", "opus".into()),
    ];
    if let Some(z) = &sticky.z {
        fields.push(("z", format!("{z}V").into()));
    }
    let prelim: MapPrelim = fields.into_iter().collect();
    Box::new(move |txn, shapes| {
        shapes.insert(txn, id, prelim);
        Ok(())
    })
}
