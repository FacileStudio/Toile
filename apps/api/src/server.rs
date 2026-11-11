//! Boot and the serve loop.

use std::net::SocketAddr;
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;

use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use tokio_util::sync::CancellationToken;

use crate::auth::Kits;
use crate::env::Config;
use crate::modules::rooms::Hub;
use crate::state::AppState;

type BootError = Box<dyn std::error::Error + Send + Sync>;

/// How long in-flight requests get after SIGINT or SIGTERM.
pub const DRAIN: Duration = Duration::from_secs(10);

fn fail(error: &dyn std::fmt::Display) -> ExitCode {
    tracing::error!(error = %error, "toile: startup failed");
    eprintln!("api: {error}");
    ExitCode::FAILURE
}

/// Runs the binary: a `migrate` subcommand, or the server until a signal.
pub async fn run() -> ExitCode {
    let config = match Config::load() {
        Ok(config) => config,
        Err(error) => return fail(&error),
    };
    let _guard = tronc::logger::init(tronc::logger::Config::from_core(&config.core));
    let pool = match PgPoolOptions::new().max_connections(20).connect(&config.core.database_url).await {
        Ok(pool) => pool,
        Err(error) => return fail(&error),
    };
    let argv: Vec<String> = std::env::args().collect();
    if let Some(result) = tronc::migrate::command(&argv, &pool, &crate::MIGRATOR).await {
        return result.map_or_else(|error| fail(&error), |()| ExitCode::SUCCESS);
    }
    match serve(config, pool).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => fail(&error),
    }
}

/// The shared state for `pool` under `config`, with the rooms of `hub`.
pub fn state(config: &Config, pool: PgPool, kits: &Kits, hub: Hub) -> AppState {
    AppState {
        pool,
        sessions: kits.sessions.clone(),
        issuer: Arc::from(config.porte.issuer.as_str()),
        hub,
        socket_recheck: crate::modules::rooms::SOCKET_RECHECK,
        suite: kits.suite.clone(),
    }
}

fn spawn_loops(kits: &Kits, state: &AppState, token: &CancellationToken) {
    tokio::spawn(crate::modules::boards::trash::run(state.clone(), token.clone()));
    if let Some(syncer) = kits.syncer.clone() {
        let token = token.clone();
        tokio::spawn(async move { syncer.run(token).await });
    }
    if let Some(deletions) = kits.deletions.clone() {
        let token = token.clone();
        tokio::spawn(async move { deletions.run(token).await });
    }
}

async fn signal() {
    let mut term = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).ok();
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {}
        _ = async { if let Some(term) = term.as_mut() { term.recv().await; } else { std::future::pending::<()>().await } } => {}
    }
}

/// Waits for SIGINT or SIGTERM, then cancels the loops and persists and closes every room
/// (members see 1013), so open sockets do not hold the drain.
async fn stopping(token: CancellationToken, hub: Hub) {
    signal().await;
    token.cancel();
    hub.shutdown().await;
}

async fn serve(config: Config, pool: PgPool) -> Result<(), BootError> {
    tronc::migrate::run(&pool, &crate::MIGRATOR).await?;
    let hub = Hub::default();
    let kits = Kits::new(&config, &pool, &hub).await?;
    kits.suite.plans.restore().await?;
    let token = CancellationToken::new();
    let state = state(&config, pool, &kits, hub.clone());
    spawn_loops(&kits, &state, &token);
    let mut http = tronc::http::Config::from_core(&config.core);
    http.security = crate::security::config(http.security, &tronc::spa::dir_from_env());
    let layers = tronc::http::layers(http)?;
    let app = crate::router::build_router(state, &kits)?.layer(layers);
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", config.core.port)).await?;
    tracing::info!(port = config.core.port, "toile: listening");
    let server = axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(stopping(token.clone(), hub.clone()));
    let drained = async {
        token.cancelled().await;
        tokio::time::sleep(DRAIN).await;
    };
    tokio::select! {
        result = server => result?,
        () = drained => tracing::warn!("toile: drain timed out"),
    }
    hub.shutdown().await;
    Ok(())
}
