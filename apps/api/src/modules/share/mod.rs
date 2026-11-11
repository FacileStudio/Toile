//! Share links: view, comment or edit access to one board for anyone holding the token, until
//! revoked. Only the token's hash is stored.

pub mod guest;
mod router;
pub mod service;

pub use router::{public_routes, routes};
