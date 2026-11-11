//! The board document, schema 1 (`docs/board-schema.md`): its roots, its validator and its
//! starting state.

mod router;
pub mod schema;
mod shape;
mod values;

pub use router::routes;
use yrs::updates::decoder::Decode;
use yrs::{Doc, Map, ReadTxn, StateVector, Transact, Update};

/// The root map holding `{ schema: 1 }`.
pub const META: &str = "meta";
/// The root map of shapes, by id.
pub const SHAPES: &str = "shapes";
/// The schema version this server reads and writes.
pub const SCHEMA: i64 = 1;

/// A failure to read stored or sent Yjs bytes.
pub type DecodeError = Box<dyn std::error::Error + Send + Sync>;

/// An empty board document with both roots typed as maps.
pub fn new_doc() -> Doc {
    let doc = Doc::new();
    doc.get_or_insert_map(META);
    doc.get_or_insert_map(SHAPES);
    doc
}

/// A board document holding `state` then every update of `updates`, in order.
pub fn from_state<'a>(state: &[u8], updates: impl IntoIterator<Item = &'a [u8]>) -> Result<Doc, DecodeError> {
    let doc = new_doc();
    {
        let mut txn = doc.transact_mut();
        if !state.is_empty() {
            txn.apply_update(Update::decode_v1(state)?)?;
        }
        for update in updates {
            txn.apply_update(Update::decode_v1(update)?)?;
        }
    }
    Ok(doc)
}

/// The whole state of `doc` as one Yjs v1 update.
pub fn encode(doc: &Doc) -> Vec<u8> {
    doc.transact().encode_state_as_update_v1(&StateVector::default())
}

/// The state a new board starts from: `meta.schema = 1` and no shapes.
pub fn seed() -> Vec<u8> {
    let doc = new_doc();
    let meta = doc.get_or_insert_map(META);
    meta.insert(&mut doc.transact_mut(), "schema", SCHEMA);
    encode(&doc)
}

#[cfg(test)]
mod tests;
