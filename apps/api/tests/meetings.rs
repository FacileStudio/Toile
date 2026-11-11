mod common;

use axum::body::Body;
use http::{Request, header};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

async fn meet(app: &common::App, who: &common::Person, body: Value) -> (u16, Value) {
    let answer = app.call("POST", "/api/boards/meeting", Some(who), Some(body)).await;
    (answer.status.as_u16(), answer.json())
}

#[tokio::test]
async fn an_echo_room_gets_one_board_and_one_edit_link_per_person() {
    let Some(pool) = common::pool("toile_meetings").await else { return };
    let app = common::app(pool).await;
    let (host, other) = (app.person("host").await, app.person("other").await);
    let (status, first) = meet(&app, &host, json!({"room": "echo-42", "name": "Weekly"})).await;
    assert_eq!(
        (status, first["board"]["name"].clone(), first["share"]["role"].clone()),
        (201, json!("Weekly"), json!("edit"))
    );
    let (status, again) = meet(&app, &host, json!({"room": " echo-42 "})).await;
    assert_eq!(
        (status, again["board"]["id"].clone(), again["share"].clone()),
        (200, first["board"]["id"].clone(), first["share"].clone())
    );
    let token = first["share"]["url"].as_str().unwrap().strip_prefix("https://toile.test/s/").unwrap().to_owned();
    let shared = app.call("GET", &format!("/api/share/{token}"), None, None).await.json();
    assert_eq!((shared["role"].clone(), shared["board"]["id"].clone()), (json!("edit"), first["board"]["id"].clone()));
    let (_, theirs) = meet(&app, &other, json!({"room": "echo-42"})).await;
    assert_ne!(theirs["board"]["id"], first["board"]["id"]);
}

#[tokio::test]
async fn a_trashed_meeting_board_is_replaced_and_bad_rooms_are_refused() {
    let Some(pool) = common::pool("toile_meetings_trash").await else { return };
    let app = common::app(pool).await;
    let host = app.person("host").await;
    let (_, first) = meet(&app, &host, json!({"room": "r1"})).await;
    let id = first["board"]["id"].as_str().unwrap();
    assert_eq!(app.call("DELETE", &format!("/api/boards/{id}"), Some(&host), None).await.status, 204);
    let (status, next) = meet(&app, &host, json!({"room": "r1"})).await;
    assert_eq!((status, next["board"]["name"].clone()), (201, json!("Réunion")));
    assert_ne!(next["board"]["id"], first["board"]["id"]);
    assert_eq!(meet(&app, &host, json!({"room": "  "})).await.0, 400);
    assert_eq!(meet(&app, &host, json!({"room": "a\u{7}b"})).await.0, 400);
    assert_eq!(app.call("POST", "/api/boards/meeting", None, Some(json!({"room": "x"}))).await.status.as_u16(), 401);
}

fn echo(method: &str, cookie: Option<&str>) -> Request<Body> {
    let mut request = Request::builder()
        .method(method)
        .uri("/api/boards/meeting")
        .header(header::HOST, "toile.test")
        .header(header::ORIGIN, "https://echo.facile.studio");
    if let Some(token) = cookie {
        request = request
            .header(header::COOKIE, format!("session={token}"))
            .header("x-facile-csrf", "1")
            .header(header::CONTENT_TYPE, "application/json");
    }
    let body = if cookie.is_some() { Body::from(r#"{"room":"echo-slug"}"#) } else { Body::empty() };
    request.body(body).unwrap()
}

#[tokio::test]
async fn echo_preflights_then_posts_with_the_cookie_and_other_origins_get_no_cors() {
    let Some(pool) = common::pool("toile_meetings_cors").await else { return };
    let app = common::app(pool).await;
    let host = app.person("host").await;
    let router = api::cors::meetings(app.router.clone());
    let preflight = router.clone().oneshot(echo("OPTIONS", None)).await.unwrap();
    let headers = preflight.headers();
    assert_eq!(preflight.status(), 204);
    assert_eq!(headers[header::ACCESS_CONTROL_ALLOW_ORIGIN], "https://echo.facile.studio");
    assert_eq!(headers[header::ACCESS_CONTROL_ALLOW_CREDENTIALS], "true");
    assert_eq!(headers[header::ACCESS_CONTROL_ALLOW_HEADERS], "content-type, x-facile-csrf");
    let posted = router.clone().oneshot(echo("POST", Some(&host.token))).await.unwrap();
    assert_eq!(posted.status(), 201);
    assert_eq!(posted.headers()[header::ACCESS_CONTROL_ALLOW_CREDENTIALS], "true");
    let body: Value = serde_json::from_slice(&posted.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert!(body["share"]["url"].as_str().unwrap().starts_with("https://toile.test/s/"));
    let mut other = echo("OPTIONS", None);
    other.headers_mut().insert(header::ORIGIN, "https://evil.test".parse().unwrap());
    let refused = router.clone().oneshot(other).await.unwrap();
    assert!(!refused.headers().contains_key(header::ACCESS_CONTROL_ALLOW_ORIGIN));
}
