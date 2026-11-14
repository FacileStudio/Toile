use axum::Router;
use axum::routing::get;
use tronc::health::{self, Check};

use crate::auth::{self, Kits};
use crate::modules::{ai, assets, board, boards, cards, comments, meetings, rooms, share, spaces, suite, tasks};
use crate::state::AppState;

async fn api_not_found() -> tronc::Error {
    tronc::Error::not_found("route not found")
}

fn protected(state: &AppState, kits: &Kits) -> Router<AppState> {
    Router::new()
        .merge(spaces::routes())
        .merge(boards::routes())
        .merge(rooms::routes())
        .merge(share::routes())
        .merge(comments::routes())
        .merge(suite::routes())
        .merge(assets::routes())
        .merge(cards::routes())
        .merge(tasks::routes())
        .merge(meetings::routes())
        .merge(ai::routes())
        .merge(crate::mcp::pending(state.confirmations.clone()))
        .route_layer(kits.sessions.require_auth())
}

fn api(state: &AppState, kits: &Kits) -> Router<AppState> {
    let mut api = Router::new()
        .merge(auth::routes(kits))
        .merge(board::routes())
        .merge(share::public_routes())
        .merge(comments::public_routes())
        .merge(assets::public_routes())
        .merge(protected(state, kits));
    if let Some(syncer) = &kits.syncer {
        api = api.route("/registre/sync", syncer.nudge_route());
    }
    if let Some(deletions) = &kits.deletions {
        api = api.route("/registre/deletions", deletions.route());
    }
    api.fallback(api_not_found)
}

/// The whole HTTP tree, before tronc's layers: probes, `/version`, `/docs`, `/mcp` when `mcp` is
/// given, `/api`, then the SPA, framable by Docs outside its confirm page (`production` drops
/// the localhost framer).
pub fn build_router(
    state: AppState,
    kits: &Kits,
    mcp: Option<tronc::mcp::App>,
    production: bool,
) -> Result<Router, String> {
    let pool = state.pool.clone();
    let mut root = health::routes(vec![Check::db(pool)])
        .route("/version", get(crate::version::handler))
        .nest("/api", api(&state, kits))
        .with_state(state.clone())
        .merge(tronc::apiref::routes(&crate::apiref::config(), crate::apiref::document())?);
    if let Some(mcp) = mcp {
        root = mcp.mount(root)?;
    }
    let dir = tronc::spa::dir_from_env();
    Ok(match tronc::spa::available(&dir) {
        true => root.fallback_service(crate::security::spa(dir, production)?),
        false => root.fallback(tronc::http::not_found),
    })
}
