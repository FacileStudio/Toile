//! Live boards: one in-memory Y.Doc per open board, hosted by salon over y-websocket, every
//! update validated against schema 1 and stored as pending updates plus periodic snapshots.

mod core;
mod host;
mod hub;
mod live;
mod load;
pub mod persist;
mod router;
mod space;

use std::time::Duration;

pub use hub::{Edit, Hub};
pub use router::routes;
pub use salon::{Caller, Who, accept};

/// How often a socket re-checks its session or share link in production.
pub const SOCKET_RECHECK: Duration = Duration::from_secs(30);

/// The comments notice: message type 100 carrying `{"type":"comments-changed"}`.
pub fn comments_changed() -> Vec<u8> {
    salon::custom(r#"{"type":"comments-changed"}"#)
}
