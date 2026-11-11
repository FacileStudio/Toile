//! Board images: uploaded into the uploader's Toile app folder in Nuage, served back through
//! Toile to anyone who may see the board.

mod router;
mod serve;
mod sniff;
mod store;
mod upload;

pub use router::{public_routes, routes};
