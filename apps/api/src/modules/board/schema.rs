//! The schema 1 validator (`docs/board-schema.md`): what a board's Y.Doc may hold.

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use yrs::types::{Event, PathSegment};
use yrs::{Any, DeepObservable, Doc, Map, MapRef, Out, ReadTxn, Transact, TransactionMut};

use super::{META, SCHEMA, SHAPES, shape};

/// The most shapes one board holds.
pub const MAX_SHAPES: u32 = 20_000;
/// The most points one ink stroke or line holds.
pub const MAX_INK_POINTS: usize = 10_000;
/// The most characters one text or string holds.
pub const MAX_TEXT: usize = 10_000;
/// The largest Yjs update one message may carry, in bytes.
pub const MAX_UPDATE: usize = 1 << 20;
/// The largest absolute value of any number.
pub const MAX_COORD: f64 = 1e7;

/// The shapes one transaction changed, as the deep observer of `shapes` saw them.
#[derive(Debug, Default)]
pub struct Touched {
    /// Something changed that names no single shape: check them all.
    pub all: bool,
    /// The ids of the shapes added, changed or removed.
    pub shapes: HashSet<Arc<str>>,
}

fn record(touched: &mut Touched, txn: &TransactionMut, event: &Event) {
    match (event.path().front(), event) {
        (Some(PathSegment::Key(id)), _) => _ = touched.shapes.insert(id.clone()),
        (None, Event::Map(map)) => touched.shapes.extend(map.keys(txn).keys().cloned()),
        _ => touched.all = true,
    }
}

/// Watches `shapes` of `doc` and collects what each transaction touched.
pub fn watch(doc: &Doc) -> Arc<Mutex<Touched>> {
    let touched = Arc::new(Mutex::new(Touched::default()));
    let sink = touched.clone();
    doc.get_or_insert_map(SHAPES).observe_deep("toile-schema", move |txn, events| {
        let mut touched = sink.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        events.iter().for_each(|event| record(&mut touched, txn, event));
    });
    touched
}

fn roots<T: ReadTxn>(txn: &T) -> Result<(), String> {
    match txn.root_refs().find(|(name, _)| *name != META && *name != SHAPES) {
        Some((name, _)) => Err(format!("unknown root {name}")),
        None => Ok(()),
    }
}

fn meta<T: ReadTxn>(txn: &T, map: &MapRef) -> Result<(), String> {
    if map.as_ref().block_len != 0 {
        return Err("meta must not hold list content".to_owned());
    }
    map.iter(txn).try_for_each(|(key, value)| match (key, value) {
        ("schema", Out::Any(Any::Number(n))) if n.as_f64() == Some(SCHEMA as f64) => Ok(()),
        ("schema", _) => Err(format!("meta.schema must be {SCHEMA}")),
        (key, _) => Err(format!("meta has an unknown key {key}")),
    })
}

/// Checks `doc` after a transaction: its roots, `meta`, the shape count, then the shapes
/// `touched` names (every shape when `touched.all`).
pub fn check(doc: &Doc, touched: &Touched) -> Result<(), String> {
    let (meta_map, shapes) = (doc.get_or_insert_map(META), doc.get_or_insert_map(SHAPES));
    let txn = doc.transact();
    roots(&txn)?;
    meta(&txn, &meta_map)?;
    if shapes.as_ref().block_len != 0 {
        return Err("shapes must not hold list content".to_owned());
    }
    if shapes.len(&txn) > MAX_SHAPES {
        return Err(format!("a board holds at most {MAX_SHAPES} shapes"));
    }
    if touched.all {
        return shapes.iter(&txn).try_for_each(|(id, value)| shape::check(&txn, id, &value));
    }
    touched.shapes.iter().try_for_each(|id| match shapes.get(&txn, id) {
        Some(value) => shape::check(&txn, id, &value),
        None => Ok(()),
    })
}

/// Checks every shape of `doc`.
pub fn check_all(doc: &Doc) -> Result<(), String> {
    check(doc, &Touched { all: true, shapes: HashSet::new() })
}
