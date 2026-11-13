use std::path::Path;

use tronc::http::{CorsConfig, SecurityConfig, Sources};

/// The origins the installed apps load the client from: Capacitor on Android and iOS, then
/// Tauri on macOS and Linux, then Tauri on Windows.
pub const NATIVE_ORIGINS: [&str; 4] =
    ["https://localhost", "capacitor://localhost", "tauri://localhost", "http://tauri.localhost"];

/// The origin of the embedded YouTube player.
pub const PLAYER: &str = "https://www.youtube-nocookie.com";

/// The origin of the suite plan catalog.
pub const CATALOG: &str = "https://api-ardoise.facile.studio";

/// The pages that may frame a board in embed mode (`/b/<id>?embed=1`): Toile itself and Docs.
pub const FRAMERS: [&str; 2] = ["'self'", "https://docs.facile.studio"];

/// The extra framer outside production, for a Docs dev server.
pub const DEV_FRAMER: &str = "http://localhost:*";

/// The CSP sources Toile's client needs on top of tronc's policy: the inline scripts
/// adapter-static writes, hashed from the build the binary serves, the suite plan catalog
/// muse's `PricingPlans` fetches, the YouTube player link cards embed, and the [`FRAMERS`] (plus
/// [`DEV_FRAMER`] outside production). Opening `frame-ancestors` drops `X-Frame-Options`.
pub fn sources(client_dir: &Path, production: bool) -> Sources {
    let mut scripts = Vec::new();
    if tronc::spa::available(client_dir) {
        match tronc::spa::inline_script_hashes(client_dir) {
            Ok(hashes) => scripts.extend(hashes),
            Err(error) => tracing::error!(%error, "failed to hash the client's inline scripts"),
        }
    }
    let mut framers: Vec<String> = FRAMERS.iter().map(|s| (*s).to_owned()).collect();
    if !production {
        framers.push(DEV_FRAMER.to_owned());
    }
    Sources::from([
        ("frame-ancestors".to_owned(), framers),
        ("script-src".to_owned(), scripts),
        ("connect-src".to_owned(), vec![CATALOG.to_owned()]),
        ("frame-src".to_owned(), vec![PLAYER.to_owned()]),
    ])
}

/// tronc's security configuration with [`sources`].
pub fn config(base: SecurityConfig, client_dir: &Path, production: bool) -> SecurityConfig {
    SecurityConfig { csp_sources: sources(client_dir, production), ..base }
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
        let sources = sources(Path::new("/nonexistent"), true);
        assert_eq!(sources.get("frame-src"), Some(&vec![PLAYER.to_owned()]));
        let framers = vec!["'self'".to_owned(), "https://docs.facile.studio".to_owned()];
        assert_eq!(sources.get("frame-ancestors"), Some(&framers));
        let dev = super::sources(Path::new("/nonexistent"), false);
        assert!(dev["frame-ancestors"].contains(&DEV_FRAMER.to_owned()));
    }

    #[test]
    fn native_origins_are_added_once() {
        let base = CorsConfig { allowed_origins: vec!["https://localhost".to_owned()], ..CorsConfig::default() };
        let origins = cors(base).allowed_origins;
        assert_eq!(origins.len(), NATIVE_ORIGINS.len());
        assert!(NATIVE_ORIGINS.iter().all(|origin| origins.iter().any(|o| o == origin)));
    }
}
