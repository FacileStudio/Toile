use super::*;
use axum::body::Body;
use http::Request;
use tower::ServiceExt;

#[test]
fn the_policy_frames_only_the_youtube_player_and_only_docs_may_frame_toile() {
    let sources = sources(Path::new("/nonexistent"));
    assert_eq!(sources.get("frame-src"), Some(&vec![PLAYER.to_owned()]));
    assert_eq!(sources.get("frame-ancestors"), None);
}

#[test]
fn only_boards_and_shares_are_embeddable_on_the_decoded_path() {
    for path in ["/b/123", "/b/x", "/s/y", "/s/y/", "/%62/x"] {
        assert!(embeddable(path), "{path}");
    }
    for path in [
        "/pending",
        "/pending/",
        "/pend%69ng",
        "/settings",
        "/boards",
        "/",
        "/b/",
        "/b/x/y",
        "/b/..%2Fpending",
        "/b/%2e%2e",
        "/s/a%2Fb",
        "/api/b/x",
    ] {
        assert!(!embeddable(path), "{path}");
    }
}

async fn framing(path: &'static str) -> (String, Option<String>) {
    let dir = std::env::temp_dir().join(format!("toile-spa-{}", uuid::Uuid::new_v4().simple()));
    std::fs::create_dir_all(&dir).expect("dir");
    std::fs::write(dir.join("index.html"), "<!doctype html><title>Toile</title>").expect("index");
    let http = tronc::http::Config { security: config(SecurityConfig::default(), &dir), ..Default::default() };
    let app = Router::new()
        .route("/api/mcp/pending", axum::routing::get(|| async { "[]" }))
        .fallback_service(spa(dir.clone(), true).expect("spa"))
        .layer(tronc::http::layers(http).expect("layers"));
    let answer = app.oneshot(Request::get(path).body(Body::empty()).expect("request")).await.expect("answer");
    std::fs::remove_dir_all(dir).expect("cleanup");
    let policy = answer.headers()["content-security-policy"].to_str().expect("policy").to_owned();
    (policy, answer.headers().get("x-frame-options").map(|v| v.to_str().expect("deny").to_owned()))
}

#[tokio::test]
async fn docs_may_frame_boards_and_shares() {
    for path in ["/b/123?embed=1", "/b/x", "/s/y?embed=1"] {
        let (policy, deny) = framing(path).await;
        assert!(policy.contains("frame-ancestors 'self' https://docs.facile.studio;"), "{path}: {policy}");
        assert!(deny.is_none(), "{path}");
    }
}

#[tokio::test]
async fn nothing_frames_the_confirm_page_the_other_pages_or_the_api() {
    for path in ["/pending?action=a1", "/pending/", "/pend%69ng", "/settings", "/boards", "/", "/api/mcp/pending"] {
        let (policy, deny) = framing(path).await;
        assert!(policy.contains("frame-ancestors 'none'"), "{path}: {policy}");
        assert_eq!(deny.as_deref(), Some("DENY"), "{path}");
    }
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
