//! Boards: the canvases of a scope, personal or a Registre space, and their trash.

pub mod access;
mod handlers;
mod router;
pub mod service;
mod thumbnail;
pub mod trash;
pub mod types;

pub use router::routes;
