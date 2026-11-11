mod common;

use std::time::Duration;

use common::board::ws;
use common::fixture;
use common::ysync::Peer;
use serde_json::json;

const QUIET: Duration = Duration::from_millis(400);

async fn thread(app: &common::App, who: &common::Person, board: &str, anchor: serde_json::Value) -> serde_json::Value {
    let body = Some(json!({"anchor": anchor, "body": "  Why?  "}));
    let answer = app.call("POST", &format!("/api/boards/{board}/comments"), Some(who), body).await;
    assert_eq!(answer.status, 201, "{}", answer.text());
    answer.json()
}

#[tokio::test]
async fn a_thread_and_its_reply_tell_open_clients_with_type_100() {
    let Some(pool) = common::pool("toile_comments").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let board = fixture::id(&fixture::board(&app, &roles.owner, Some(&roles.space), "Talk").await);
    let mut open = Peer::new();
    open.connect(&ws(&app.serve().await, &board), Some(&roles.admin.token)).await;
    let anchor = json!({"shape": "s1", "x": 10.5, "y": -3.5});
    let thread = thread(&app, &roles.member, &board, anchor.clone()).await;
    let summary = (thread["body"].clone(), thread["mine"].clone(), thread["guest"].clone(), thread["author"].clone());
    assert_eq!(summary, (json!("Why?"), json!(true), json!(false), json!("member")));
    let path = format!("/api/boards/{board}/comments");
    let reply = json!({"body": "Because", "parent": thread["id"]});
    let reply = app.call("POST", &path, Some(&roles.owner), Some(reply)).await.json();
    assert_eq!((reply["anchor"].clone(), reply["parent"].clone()), (anchor, thread["id"].clone()));
    let nested = json!({"body": "deeper", "parent": reply["id"]});
    assert_eq!(app.call("POST", &path, Some(&roles.owner), Some(nested)).await.status, 400);
    open.pump(QUIET).await;
    assert_eq!(open.customs, vec![(100u8, br#"{"type":"comments-changed"}"#.to_vec()); 2]);
    assert_eq!(app.call("GET", &path, Some(&roles.outsider), None).await.status, 404);
}

#[tokio::test]
async fn only_an_author_edits_any_commenter_resolves_and_an_owner_deletes() {
    let Some(pool) = common::pool("toile_comments_rights").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let board = fixture::id(&fixture::board(&app, &roles.owner, Some(&roles.space), "Talk").await);
    let id = thread(&app, &roles.member, &board, json!({"x": 0, "y": 0})).await["id"].as_str().unwrap().to_owned();
    let (path, one) = (format!("/api/boards/{board}/comments"), format!("/api/boards/{board}/comments/{id}"));
    assert_eq!(app.call("PATCH", &one, Some(&roles.owner), Some(json!({"body": "edited"}))).await.status, 403);
    let resolved = app.call("PATCH", &one, Some(&roles.owner), Some(json!({"resolved": true}))).await.json();
    assert!(resolved["resolved_at"].is_string());
    let edited = app.call("PATCH", &one, Some(&roles.member), Some(json!({"body": "Why not?"}))).await.json();
    assert_eq!((edited["body"].clone(), edited["mine"].clone()), (json!("Why not?"), json!(true)));
    assert!(edited["resolved_at"].is_string());
    let mine = thread(&app, &roles.owner, &board, json!({"x": 1, "y": 1})).await;
    let theirs = format!("{path}/{}", mine["id"].as_str().unwrap());
    assert_eq!(app.call("DELETE", &theirs, Some(&roles.member), None).await.status, 403);
    assert_eq!(app.call("DELETE", &one, Some(&roles.outsider), None).await.status, 404);
    assert_eq!(app.call("DELETE", &one, Some(&roles.admin), None).await.status, 204);
    assert_eq!(app.call("DELETE", &theirs, Some(&roles.owner), None).await.status, 204);
    assert_eq!(app.call("GET", &path, Some(&roles.owner), None).await.json(), json!([]));
}

#[tokio::test]
async fn bad_comment_bodies_and_anchors_are_refused() {
    let Some(pool) = common::pool("toile_comments_bad").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let board = fixture::id(&fixture::board(&app, &roles.member, None, "Bad").await);
    let path = format!("/api/boards/{board}/comments");
    let bodies = [
        json!({"body": "no anchor"}),
        json!({"anchor": {"x": 1, "y": 1}, "body": "   "}),
        json!({"anchor": {"x": 1e8, "y": 1}, "body": "far"}),
        json!({"anchor": {"x": 1, "y": 1, "z": 2}, "body": "unknown"}),
        json!({"anchor": {"x": 1, "y": 1}, "body": "x".repeat(10_001)}),
        json!({"anchor": {"x": 1, "y": 1}, "body": "x", "parent": uuid::Uuid::new_v4()}),
    ];
    for body in bodies {
        let status = app.call("POST", &path, Some(&roles.member), Some(body.clone())).await.status;
        assert_eq!(status.as_u16() / 100, 4, "{body}");
    }
}

#[tokio::test]
async fn guests_comment_by_name_through_comment_and_edit_links_only() {
    let Some(pool) = common::pool("toile_comments_guest").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let board = fixture::id(&fixture::board(&app, &roles.owner, None, "Open").await);
    let body = |name: &str| json!({"name": name, "anchor": {"x": 0, "y": 0}, "body": "hello"});
    for (role, status) in [("view", 403), ("comment", 201), ("edit", 201)] {
        let (_, token) = fixture::link(&app, &roles.owner, &board, role).await;
        let path = format!("/api/share/{token}/comments");
        assert_eq!(app.call("POST", &path, None, Some(body(" Zoé "))).await.status, status, "{role}");
        assert_eq!(app.call("GET", &path, None, None).await.status, 200);
    }
    let (_, token) = fixture::link(&app, &roles.owner, &board, "comment").await;
    let path = format!("/api/share/{token}/comments");
    assert_eq!(app.call("POST", &path, None, Some(body(""))).await.status, 400);
    let listed = app.call("GET", &format!("/api/boards/{board}/comments"), Some(&roles.owner), None).await.json();
    assert_eq!(listed.as_array().unwrap().len(), 2);
    assert_eq!((listed[0]["author"].clone(), listed[0]["guest"].clone()), (json!("Zoé"), json!(true)));
    let mut statuses = Vec::new();
    for _ in 0..12 {
        statuses.push(app.call("POST", &path, None, Some(body("Spam"))).await.status.as_u16());
    }
    assert_eq!(*statuses.last().unwrap(), 429, "{statuses:?}");
}
