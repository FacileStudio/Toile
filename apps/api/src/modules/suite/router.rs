use std::collections::BTreeMap;

use axum::Router;
use axum::extract::{Request, State};
use axum::middleware::{self, Next};
use axum::response::Response;
use axum::routing::get;
use porte::Identity;
use porte::session::Manager;
use serde::{Deserialize, Serialize};
use tronc::Result;
use tronc::json::Json;

use super::{AI_ACCESS, Suite, UPLOAD_MB, scope};
use crate::extract::Params;
use crate::modules::boards::access;
use crate::state::AppState;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Query {
    space_id: Option<String>,
}

#[derive(Serialize)]
struct Entitlements {
    scope: &'static str,
    entitlements: BTreeMap<&'static str, serde_json::Value>,
    upgrade_url: String,
}

async fn entitlements(
    State(state): State<AppState>,
    who: Identity,
    Params(query): Params<Query>,
) -> Result<Json<Entitlements>> {
    let space_id = query.space_id.as_deref();
    access::scope(&state, who.user_id, space_id).await?;
    let plan = scope(&state, who.user_id, space_id).await?;
    let plans = &state.suite.plans;
    let entitlements = BTreeMap::from([
        (AI_ACCESS, plans.enabled(&plan, AI_ACCESS).into()),
        (UPLOAD_MB, plans.limit(&plan, UPLOAD_MB).into()),
    ]);
    Ok(Json(Entitlements {
        scope: if space_id.is_some() { "space" } else { "personal" },
        entitlements,
        upgrade_url: porte::forfait::upgrade_url(&state.issuer, space_id.unwrap_or_default()),
    }))
}

async fn forget(State(suite): State<Suite>, request: Request, next: Next) -> Response {
    let user_id = request.extensions().get::<Identity>().map(|identity| identity.user_id);
    let response = next.run(request).await;
    if let Some(user_id) = user_id.filter(|_| response.status().is_success()) {
        suite.forget_user(user_id);
    }
    response
}

/// `GET /me/entitlements?space_id=`, behind `require_auth`: the plan values of the caller's
/// personal scope or of one of their spaces, for the plan page.
pub fn routes() -> Router<AppState> {
    Router::new().route("/me/entitlements", get(entitlements))
}

/// porte's `POST /auth/logout`, which also drops the person's exchanged tokens once it succeeds.
pub fn logout_routes<S: Clone + Send + Sync + 'static>(sessions: &Manager, suite: Suite) -> Router<S> {
    sessions.routes().layer(middleware::from_fn_with_state(suite, forget)).layer(sessions.optional())
}
