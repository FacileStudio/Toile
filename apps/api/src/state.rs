use std::sync::Arc;
use std::time::Duration;

use porte::pg::TokenSealer;
use porte::session::Manager;
use sqlx::PgPool;
use tronc::mcp::Confirmations;

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
    /// The MCP confirmation flow, when `/mcp` is on.
    pub confirmations: Option<Confirmations>,
    /// Seals the meeting links Toile keeps to answer again; `None` keeps them plain (dev).
    pub sealer: Option<Arc<TokenSealer>>,
    /// The public origin, `https://toile.facile.studio`, from `OIDC_REDIRECT_URL`; empty in dev.
    pub public_url: Arc<str>,
}
