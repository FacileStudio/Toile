//! Board import: JSON Canvas and Excalidraw files become a new schema-1 board.

pub mod build;
pub mod elements;
pub mod excalidraw;
pub mod jsoncanvas;
mod router;
pub mod service;

pub use router::routes;
