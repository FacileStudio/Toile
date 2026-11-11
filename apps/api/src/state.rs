use std::sync::Arc;

use sqlx::PgPool;

/// What every handler shares.
#[derive(Clone)]
pub struct AppState {
    /// The database.
    pub pool: PgPool,
    /// The Registre issuer: the `provider` of every identity whose subject is a space member.
    pub issuer: Arc<str>,
}
