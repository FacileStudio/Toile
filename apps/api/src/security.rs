use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::Router;
use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;
use tronc::http::{CorsConfig, SecurityConfig, SecurityOverride, SecurityOverrideConfig, Sources, security_override};

/// The origins the installed apps load the client from: Capacitor on Android and iOS, then
/// Tauri on macOS and Linux, then Tauri on Windows.
pub const NATIVE_ORIGINS: [&str; 4] =
    ["https://localhost", "capacitor://localhost", "tauri://localhost", "http://tauri.localhost"];

/// The origin of the embedded YouTube player.
pub const PLAYER: &str = "https://www.youtube-nocookie.com";

/// The origin of the suite plan catalog.
pub const CATALOG: &str = "https://api-ardoise.facile.studio";

/// The origin of the Vision analytics tracker `app.html` loads on the production host. Its
/// pixels are https images, which tronc's default `img-src` already allows.
pub const ANALYTICS: &str = "https://vision.facile.studio";

/// The pages that may frame a client page Docs embeds ([`embeddable`]): Toile itself and Docs.
pub const FRAMERS: [&str; 2] = ["'self'", "https://docs.facile.studio"];

/// The extra framer outside production, for a Docs dev server.
pub const DEV_FRAMER: &str = "http://localhost:*";

/// The CSP sources Toile's client needs on top of tronc's policy: the inline scripts
/// adapter-static writes, hashed from the build the binary serves, the Vision tracker, the suite
/// plan catalog muse's `PricingPlans` fetches and the YouTube player link cards embed. Every
/// response keeps tronc's `frame-ancestors 'none'`; [`spa`] opens it for the client's pages.
pub fn sources(client_dir: &Path) -> Sources {
    let mut scripts = vec![ANALYTICS.to_owned()];
    if tronc::spa::available(client_dir) {
        match tronc::spa::inline_script_hashes(client_dir) {
            Ok(hashes) => scripts.extend(hashes),
            Err(error) => tracing::error!(%error, "failed to hash the client's inline scripts"),
        }
    }
    Sources::from([
        ("script-src".to_owned(), scripts),
        ("connect-src".to_owned(), vec![CATALOG.to_owned()]),
        ("frame-src".to_owned(), vec![PLAYER.to_owned()]),
    ])
}

/// tronc's security configuration with [`sources`].
pub fn config(base: SecurityConfig, client_dir: &Path) -> SecurityConfig {
    SecurityConfig { csp_sources: sources(client_dir), ..base }
}

/// Whether `path` is a client page Docs embeds: a board (`/b/<id>`) or a shared board
/// (`/s/<token>`), decided on the percent-decoded path with at most one trailing slash. Every
/// other page, the MCP confirm page first, keeps `frame-ancestors 'none'`.
pub fn embeddable(path: &str) -> bool {
    let decoded = percent_encoding::percent_decode_str(path).decode_utf8_lossy();
    let path = decoded.strip_suffix('/').unwrap_or(&decoded);
    let segments: Vec<&str> = path.split('/').collect();
    matches!(segments.as_slice(), ["", "b" | "s", id] if !id.is_empty() && *id != "." && *id != "..")
}

async fn frame_embeds(State(framing): State<Arc<SecurityOverride>>, request: Request, next: Next) -> Response {
    if embeddable(request.uri().path()) {
        security_override(State(framing), request, next).await
    } else {
        next.run(request).await
    }
}

/// The built client at `dir`, answering every path the other routes leave. Only the
/// [`embeddable`] pages are framable, by the [`FRAMERS`] (plus [`DEV_FRAMER`] outside
/// production); every other page, the MCP confirm page included, keeps `frame-ancestors 'none'`
/// so no framing page can trick a click on its buttons.
pub fn spa(dir: PathBuf, production: bool) -> Result<Router, String> {
    let mut framers: Vec<String> = FRAMERS.iter().map(|s| (*s).to_owned()).collect();
    if !production {
        framers.push(DEV_FRAMER.to_owned());
    }
    let framing = SecurityOverride::new(SecurityOverrideConfig {
        csp_sources: Sources::from([("frame-ancestors".to_owned(), framers)]),
        ..SecurityOverrideConfig::default()
    })?;
    let handler = tronc::spa::handler(tronc::spa::Config { dir, ..Default::default() });
    Ok(Router::new()
        .fallback_service(handler)
        .layer(axum::middleware::from_fn_with_state(Arc::new(framing), frame_embeds)))
}

/// `base` with [`NATIVE_ORIGINS`] appended once. The apps send a bearer token and no cookie, so
/// credentials stay off.
pub fn cors(base: CorsConfig) -> CorsConfig {
    let mut origins = base.allowed_origins;
    for origin in NATIVE_ORIGINS {
        if !origins.iter().any(|known| known == origin) {
            origins.push(origin.to_owned());
        }
    }
    CorsConfig { allowed_origins: origins, ..base }
}

#[cfg(test)]
mod tests;
