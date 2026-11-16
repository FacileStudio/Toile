//! The board document: planche schema 1 under its board profile (`docs/board-schema.md`), and
//! `GET /board-schema`.

mod router;

pub use planche::doc::{DecodeError, encode, from_state, new_doc, seed};
pub use planche::{META, SCHEMA, SHAPES};
pub use router::routes;
