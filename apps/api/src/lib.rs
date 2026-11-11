//! Toile, the Facile suite's collaborative infinite canvas: the API.
#![deny(unused, dead_code)]

pub mod apiref;
pub mod auth;
pub mod env;
pub mod extract;
pub mod modules;
pub mod router;
pub mod security;
pub mod server;
pub mod state;
pub mod version;

/// The app's migrations, embedded at build time.
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");
