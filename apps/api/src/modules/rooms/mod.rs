//! Live boards: one in-memory Y.Doc per open board, spoken to over y-websocket, every update
//! validated against schema 1 and stored as pending updates plus periodic snapshots.

mod command;
mod core;
mod drain;
mod hub;
mod load;
mod members;
pub mod persist;
mod presence;
mod room;
mod router;
mod sync;
mod writes;
pub mod ws;

pub use command::{Command, Edit, Who};
pub use hub::Hub;
pub use router::{accept, routes};
pub use sync::comments_changed;
pub use ws::{Caller, SOCKET_RECHECK};
