use axum::Router;
use axum::routing::get;
use tronc::health::{self, Check};

use crate::auth::{self, Kits};
use crate::modules::{assets, board, boards, cards, comments, meetings, pending, rooms, share, spaces, suite, tasks};
use crate::state::AppState;

async fn api_not_found() -> tronc::Error {
    tronc::Error::not_found("route not found")
}

fn protected(kits: &Kits) -> Router<AppState> {
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
        .merge(pending::routes())
        .route_layer(kits.sessions.require_auth())
}

fn api(kits: &Kits) -> Router<AppState> {
    let mut api = Router::new()
        .merge(auth::routes(kits))
        .merge(board::routes())
        .merge(share::public_routes())
        .merge(comments::public_routes())
        .merge(assets::public_routes())
        .merge(protected(kits));
    if let Some(syncer) = &kits.syncer {
        api = api.route("/registre/sync", syncer.nudge_route());
    }
    if let Some(deletions) = &kits.deletions {
        api = api.route("/registre/deletions", deletions.route());
    }
    api.fallback(api_not_found)
}

/// The whole HTTP tree, before tronc's layers: probes, `/version`, `/docs`, `/mcp` when `mcp` is
/// given, `/api`, then the SPA.
pub fn build_router(state: AppState, kits: &Kits, mcp: Option<tronc::mcp::Options>) -> Result<Router, String> {
    let pool = state.pool.clone();
    let mut root = health::routes(vec![Check::db(pool)])
        .route("/version", get(crate::version::handler))
        .nest("/api", api(kits))
        .with_state(state.clone())
        .merge(tronc::apiref::routes(&crate::apiref::config(), crate::apiref::document())?);
    if let (Some(options), Some(confirmations)) = (mcp, state.confirmations.clone()) {
        root = crate::mcp::mount(root, &state, options, confirmations)?;
    }
    let dir = tronc::spa::dir_from_env();
    Ok(match tronc::spa::available(&dir) {
        true => root.fallback_service(tronc::spa::handler(tronc::spa::Config { dir, ..Default::default() })),
        false => root.fallback(tronc::http::not_found),
    })
}
