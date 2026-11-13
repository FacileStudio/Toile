//! AI on a board, through Bulle: summarize it, sort selected stickies into framed themes, and
//! draft a mind map, a flowchart or stickies from a prompt. Every route needs editor access and
//! a plan with `ai.access`; a plan without it answers `402` before Bulle is called. Writes go
//! through the board's room, so open clients see them live. The caller's assistant name needs
//! editor access only.

mod assistant;
mod bulle;
mod cluster;
mod generate;
mod router;
mod shapes;
mod summarize;

use axum::Router;

use crate::state::AppState;

/// The AI routes and the assistant name route, behind `require_auth`.
pub fn routes() -> Router<AppState> {
    router::routes().merge(assistant::routes())
}
