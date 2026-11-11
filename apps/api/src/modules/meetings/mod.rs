//! Echo meeting boards: one board per (person, Echo room), with an edit link for the room's
//! participants. `POST /boards/meeting` is idempotent per (caller, room).

mod router;
pub mod service;

pub use router::routes;
