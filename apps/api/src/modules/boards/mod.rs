//! Boards: the canvases of a scope, personal or a Registre space, and their trash.

pub mod access;
mod handlers;
mod router;
pub mod service;
pub mod trash;
pub mod types;

pub use router::routes;
