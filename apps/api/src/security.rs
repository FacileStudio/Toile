use std::path::Path;

use tronc::http::{CorsConfig, SecurityConfig, Sources};

/// The origins the installed apps load the client from: Capacitor on Android and iOS, then
/// Tauri on macOS and Linux, then Tauri on Windows.
pub const NATIVE_ORIGINS: [&str; 4] =
    ["https://localhost", "capacitor://localhost", "tauri://localhost", "http://tauri.localhost"];

/// The origin of the suite plan catalog.
pub const CATALOG: &str = "https://api-ardoise.facile.studio";

/// The CSP sources Toile's client needs on top of tronc's policy: the inline scripts
/// adapter-static writes, hashed from the build the binary serves, and the suite plan catalog
/// muse's `PricingPlans` fetches.
pub fn sources(client_dir: &Path) -> Sources {
    let mut scripts = Vec::new();
    if tronc::spa::available(client_dir) {
        match tronc::spa::inline_script_hashes(client_dir) {
            Ok(hashes) => scripts.extend(hashes),
            Err(error) => tracing::error!(%error, "failed to hash the client's inline scripts"),
        }
    }
    Sources::from([("script-src".to_owned(), scripts), ("connect-src".to_owned(), vec![CATALOG.to_owned()])])
}

/// tronc's security configuration with [`sources`].
pub fn config(base: SecurityConfig, client_dir: &Path) -> SecurityConfig {
    SecurityConfig { csp_sources: sources(client_dir), ..base }
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
    fn native_origins_are_added_once() {
        let base = CorsConfig { allowed_origins: vec!["https://localhost".to_owned()], ..CorsConfig::default() };
        let origins = cors(base).allowed_origins;
        assert_eq!(origins.len(), NATIVE_ORIGINS.len());
        assert!(NATIVE_ORIGINS.iter().all(|origin| origins.iter().any(|o| o == origin)));
    }
}
