//! The rest of the suite: token exchange to other apps and the plan entitlements.

mod apps;
mod call;
mod plans;
mod router;

use porte::exchange::Exchanger;
use porte::forfait::Scope;
use tronc::Result;

pub use apps::{App, Apps};
pub use call::client;
pub use plans::{AI_ACCESS, Plans, UPLOAD_MB};
pub use router::{logout_routes, routes};

use crate::extract::db;
use crate::state::AppState;

/// What Toile needs to act in other suite apps for a signed-in person.
#[derive(Clone)]
pub struct Suite {
    /// Mints the person's tokens for other apps; `None` without Registre.
    pub exchanger: Option<Exchanger>,
    /// The plan values from the directory snapshot.
    pub plans: Plans,
    /// The suite apps Toile calls.
    pub apps: Apps,
    /// The client those calls go through.
    pub http: reqwest::Client,
    /// Resolved link cards and the unfurl client.
    pub cards: crate::modules::cards::Cards,
}

impl Suite {
    /// Drops every exchanged token held for `user_id`.
    pub fn forget_user(&self, user_id: i64) {
        if let Some(exchanger) = &self.exchanger {
            exchanger.forget_user(user_id);
        }
    }
}

/// The plan scope of something in `space_id`, or of the person's own boards: the space's plan,
/// or the plan of the person's Registre subject (the most recently synced one under the issuer).
pub async fn scope(state: &AppState, user_id: i64, space_id: Option<&str>) -> Result<Scope> {
    if let Some(space_id) = space_id {
        return Ok(Scope::space(space_id));
    }
    let subject: Option<String> = sqlx::query_scalar(
        "SELECT subject FROM porte_identities WHERE user_id = $1 AND provider = $2
         ORDER BY synced_at DESC NULLS LAST, created_at DESC NULLS LAST LIMIT 1",
    )
    .bind(user_id)
    .bind(&*state.issuer)
    .fetch_optional(&state.pool)
    .await
    .map_err(db)?;
    Ok(Scope::person(subject.unwrap_or_default()))
}
