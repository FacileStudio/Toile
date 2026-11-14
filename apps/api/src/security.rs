use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::Router;
use tronc::http::{CorsConfig, SecurityConfig, SecurityOverride, SecurityOverrideConfig, Sources, security_override};
use tronc::mcp::PENDING_PATH;

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

/// The pages that may frame the client, for a board in embed mode (`/b/<id>?embed=1`): Toile
/// itself and Docs.
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

/// The built client at `dir`, answering every path the other routes leave, framable by the
/// [`FRAMERS`] (plus [`DEV_FRAMER`] outside production) except the MCP confirm page, which keeps
/// `frame-ancestors 'none'` so no framing page can trick a click on its confirm button.
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
    let framable = Router::new()
        .fallback_service(handler.clone())
        .layer(axum::middleware::from_fn_with_state(Arc::new(framing), security_override));
    Ok(Router::new().route_service(PENDING_PATH, handler).fallback_service(framable))
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
mod tests {
    use super::*;

    #[test]
    fn the_policy_frames_only_the_youtube_player_and_only_docs_may_frame_toile() {
        let sources = sources(Path::new("/nonexistent"));
        assert_eq!(sources.get("frame-src"), Some(&vec![PLAYER.to_owned()]));
        assert_eq!(sources.get("frame-ancestors"), None);
    }

    #[tokio::test]
    async fn docs_may_frame_the_client_but_not_the_confirm_page_or_the_api() {
        use axum::body::Body;
        use http::Request;
        use tower::ServiceExt;

        let dir = std::env::temp_dir().join(format!("toile-spa-{}", uuid::Uuid::new_v4().simple()));
        std::fs::create_dir_all(&dir).expect("dir");
        std::fs::write(dir.join("index.html"), "<!doctype html><title>Toile</title>").expect("index");
        let http = tronc::http::Config { security: config(SecurityConfig::default(), &dir), ..Default::default() };
        let app = Router::new()
            .route("/api/mcp/pending", axum::routing::get(|| async { "[]" }))
            .fallback_service(spa(dir.clone(), true).expect("spa"))
            .layer(tronc::http::layers(http).expect("layers"));
        let framing = |path: &'static str| {
            let app = app.clone();
            async move {
                let answer =
                    app.oneshot(Request::get(path).body(Body::empty()).expect("request")).await.expect("answer");
                let policy = answer.headers()["content-security-policy"].to_str().expect("policy").to_owned();
                (policy, answer.headers().get("x-frame-options").cloned())
            }
        };
        let (board, deny) = framing("/b/123?embed=1").await;
        assert!(board.contains("frame-ancestors 'self' https://docs.facile.studio;"), "{board}");
        assert!(deny.is_none());
        for path in ["/pending?action=a1", "/api/mcp/pending"] {
            let (policy, deny) = framing(path).await;
            assert!(policy.contains("frame-ancestors 'none'"), "{path}: {policy}");
            assert_eq!(deny.as_ref().map(|v| v.to_str().unwrap()), Some("DENY"), "{path}");
        }
        std::fs::remove_dir_all(dir).expect("cleanup");
    }

    #[test]
    fn the_policy_loads_the_vision_tracker() {
        let sources = sources(Path::new("/nonexistent"));
        assert_eq!(sources.get("script-src"), Some(&vec![ANALYTICS.to_owned()]));
    }

    #[test]
    fn native_origins_are_added_once() {
        let base = CorsConfig { allowed_origins: vec!["https://localhost".to_owned()], ..CorsConfig::default() };
        let origins = cors(base).allowed_origins;
        assert_eq!(origins.len(), NATIVE_ORIGINS.len());
        assert!(NATIVE_ORIGINS.iter().all(|origin| origins.iter().any(|o| o == origin)));
    }
}
