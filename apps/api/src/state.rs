use std::sync::Arc;
use std::time::Duration;

use porte::session::Manager;
use sqlx::PgPool;

use crate::modules::rooms::Hub;
use crate::modules::suite::Suite;

/// What every handler shares.
#[derive(Clone)]
pub struct AppState {
    /// The database.
    pub pool: PgPool,
    /// Sessions, for the WebSocket origin check.
    pub sessions: Manager,
    /// The Registre issuer: the `provider` of every identity whose subject is a space member.
    pub issuer: Arc<str>,
    /// The open board rooms.
    pub hub: Hub,
    /// How often an open socket re-checks its session or share link.
    pub socket_recheck: Duration,
    /// Token exchange and plans.
    pub suite: Suite,
}
