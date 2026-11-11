//! A sticky turned into an Opus task: the task is created in Opus for the editor, and a card
//! linked to it is written beside the sticky through the board's room.

mod opus;
mod router;
mod sticky;

pub use router::routes;
