//! AI on a board, through Bulle: summarize it, sort selected stickies into framed themes, and
//! draft a mind map, a flowchart or stickies from a prompt. Every route needs editor access and
//! a plan with `ai.access`; a plan without it answers `402` before Bulle is called. Writes go
//! through the board's room, so open clients see them live.

mod bulle;
mod cluster;
mod generate;
mod router;
mod shapes;
mod summarize;

pub use router::routes;
