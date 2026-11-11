use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use yrs::types::text::TextPrelim;
use yrs::{Any, Doc, In, Map, MapPrelim, MapRef, Transact, TransactionMut};

use crate::modules::board;
use crate::modules::export::model::{Binding, Kind, Shape, Style};

const DIGITS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

/// The `i`-th fractional index, in the `a0`, `b00`, `c000` integer-key form the canvas
/// client's `fractional-indexing` reads. Keys sort in `i` order.
pub fn z_key(i: usize) -> String {
    let (head, width, base) = match i {
        i if i < 62 => ('a', 1, 0),
        i if i < 62 + 62 * 62 => ('b', 2, 62),
        _ => ('c', 3, 62 + 62 * 62),
    };
    let mut n = i - base;
    let mut digits = vec![b'0'; width];
    for slot in digits.iter_mut().rev() {
        *slot = DIGITS[n % 62];
        n /= 62;
    }
    format!("{head}{}", String::from_utf8_lossy(&digits))
}

/// What an importer made of a file: the shapes in stacking order, and how many source
/// elements had no schema-1 equivalent.
#[derive(Debug, Default)]
pub struct Imported {
    pub shapes: Vec<Shape>,
    pub skipped: usize,
}

/// Turns the ids of an imported file into shape ids: a safe id is kept, anything else, or a
/// repeat, gets a fresh UUID. The same source id always maps to the same shape id.
#[derive(Default)]
pub struct Ids {
    known: HashMap<String, String>,
    used: HashSet<String>,
}

impl Ids {
    /// The shape id for source id `raw`.
    pub fn get(&mut self, raw: &str) -> String {
        if let Some(id) = self.known.get(raw) {
            return id.clone();
        }
        let safe = (1..=64).contains(&raw.len()) && raw.chars().all(|c| c.is_ascii_alphanumeric() || "-_".contains(c));
        let id = if safe && !self.used.contains(raw) { raw.to_owned() } else { uuid::Uuid::new_v4().to_string() };
        self.used.insert(id.clone());
        self.known.insert(raw.to_owned(), id.clone());
        id
    }
}

fn style(style: &Style) -> Any {
    let mut map = HashMap::new();
    let strings = [("fill", &style.fill), ("stroke", &style.stroke), ("font", &style.font), ("align", &style.align)];
    for (key, value) in strings {
        if let Some(value) = value {
            map.insert(key.to_owned(), Any::from(value.clone()));
        }
    }
    for (key, value) in [("width", style.width), ("size", style.size)] {
        if let Some(value) = value {
            map.insert(key.to_owned(), Any::from(value));
        }
    }
    Any::Map(Arc::new(map))
}

fn binding(binding: &Binding) -> In {
    let anchor: Vec<Any> = binding.anchor.iter().map(|v| Any::from(*v)).collect();
    let map = HashMap::from([
        ("shape".to_owned(), Any::from(binding.shape.clone())),
        ("anchor".to_owned(), Any::from(anchor)),
    ]);
    In::Any(Any::Map(Arc::new(map)))
}

fn specific(shape: &Shape, map: &mut MapPrelim) {
    let points = || In::Any(Any::from(shape.points.iter().map(|v| Any::from(*v)).collect::<Vec<_>>()));
    match shape.kind {
        k if k.has_text() => _ = map.insert("text".into(), In::Text(TextPrelim::new(shape.text.clone()).into())),
        Kind::Line => _ = map.insert("points".into(), points()),
        Kind::Arrow => {
            map.insert("points".into(), points());
            map.insert("head".into(), In::from(shape.head.clone().unwrap_or_else(|| "end".to_owned())));
            shape.from.as_ref().map(|b| map.insert("from".into(), binding(b)));
            shape.to.as_ref().map(|b| map.insert("to".into(), binding(b)));
        }
        Kind::Ink => _ = map.insert("points".into(), points()),
        Kind::Image => _ = map.insert("asset".into(), In::from(shape.asset.clone())),
        Kind::Card => {
            map.insert("url".into(), In::from(shape.url.clone()));
            map.insert("title".into(), In::from(shape.title.clone()));
            map.insert("kind".into(), In::from(shape.card_kind.clone()));
        }
        _ => _ = map.insert("name".into(), In::from(shape.name.clone())),
    }
}

fn prelim(shape: &Shape, z: String) -> MapPrelim {
    let mut map = MapPrelim::from([
        ("type", In::from(shape.kind.name())),
        ("x", In::from(shape.x)),
        ("y", In::from(shape.y)),
        ("w", In::from(shape.w)),
        ("h", In::from(shape.h)),
        ("rot", In::from(shape.rot)),
        ("z", In::from(z)),
        ("style", In::Any(style(&shape.style))),
    ]);
    if let Some(parent) = &shape.parent {
        map.insert("parent".into(), In::from(parent.clone()));
    }
    specific(shape, &mut map);
    map
}

/// Inserts `shapes` into the `shapes` root of a transaction, stacked in the order given.
pub fn fill(txn: &mut TransactionMut, root: &MapRef, shapes: &[Shape]) {
    for (i, shape) in shapes.iter().enumerate() {
        root.insert(txn, shape.id.as_str(), prelim(shape, z_key(i)));
    }
}

/// A schema-1 board document holding `shapes`, stacked in the order given.
pub fn document(shapes: &[Shape]) -> Doc {
    let doc = board::new_doc();
    let meta = doc.get_or_insert_map(board::META);
    let root = doc.get_or_insert_map(board::SHAPES);
    let mut txn = doc.transact_mut();
    meta.insert(&mut txn, "schema", board::SCHEMA);
    fill(&mut txn, &root, shapes);
    drop(txn);
    doc
}
