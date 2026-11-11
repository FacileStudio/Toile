//! Comments: threads pinned to a board point or shape, kept in Postgres, not in the Y.Doc. Every
//! change tells the board's open clients with a y-websocket message of type 100.

mod router;
pub mod service;
pub mod types;

pub use router::{public_routes, routes};
