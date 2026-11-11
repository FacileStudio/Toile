mod common;

use std::time::Duration;

use common::board::{guest, rect, ws};
use common::fixture;
use common::ysync::Peer;
use serde_json::{Value, json};

const QUIET: Duration = Duration::from_millis(400);

#[tokio::test]
async fn only_an_owner_manages_links_and_the_list_never_shows_a_token() {
    let Some(pool) = common::pool("toile_share_manage").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let board = fixture::id(&fixture::board(&app, &roles.owner, Some(&roles.space), "Shared").await);
    let path = format!("/api/boards/{board}/shares");
    let body = Some(json!({"role": "edit"}));
    assert_eq!(app.call("POST", &path, Some(&roles.member), body.clone()).await.status, 403);
    assert_eq!(app.call("POST", &path, Some(&roles.outsider), body).await.status, 404);
    assert_eq!(
        app.call("POST", &path, Some(&roles.owner), Some(json!({"role": "admin"}))).await.status.as_u16() / 100,
        4
    );
    for role in ["view", "comment", "edit"] {
        fixture::link(&app, &roles.owner, &board, role).await;
    }
    let listed = app.call("GET", &path, Some(&roles.admin), None).await.json();
    let roles_listed: Vec<&str> = listed.as_array().unwrap().iter().map(|l| l["role"].as_str().unwrap()).collect();
    assert_eq!(roles_listed.len(), 3);
    assert!(listed.as_array().unwrap().iter().all(|l| l.get("token").is_none() && l.get("url").is_none()));
}

#[tokio::test]
async fn each_link_role_resolves_and_only_edit_draws() {
    let Some(pool) = common::pool("toile_share_roles").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let board = fixture::id(&fixture::board(&app, &roles.owner, None, "Guests").await);
    let base = app.serve().await;
    let mut owner = Peer::new();
    owner.connect(&ws(&base, &board), Some(&roles.owner.token)).await;
    for (role, access, draws) in [("view", "viewer", false), ("comment", "commenter", false), ("edit", "editor", true)]
    {
        let (_, token) = fixture::link(&app, &roles.owner, &board, role).await;
        let shared = app.call("GET", &format!("/api/share/{token}"), None, None).await.json();
        assert_eq!((shared["role"].clone(), shared["board"]["role"].clone()), (json!(role), json!(access)));
        assert_eq!(shared["board"]["id"], json!(board));
        let mut visitor = Peer::new();
        visitor.connect(&guest(&base, &token, "Ana"), None).await;
        visitor.edit(|txn, shapes| rect(txn, shapes, role, 1.0)).await;
        visitor.pump(QUIET).await;
        owner.pump(QUIET).await;
        assert_eq!(visitor.closed, None, "{role}");
        assert_eq!(owner.shapes().contains(&role.to_owned()), draws, "{role}");
    }
}

#[tokio::test]
async fn a_guest_presence_carries_its_name_and_guest_flag_whatever_it_claims() {
    let Some(pool) = common::pool("toile_share_presence").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let board = fixture::id(&fixture::board(&app, &roles.owner, None, "Faces").await);
    let base = app.serve().await;
    let (_, token) = fixture::link(&app, &roles.owner, &board, "view").await;
    let mut owner = Peer::new();
    owner.connect(&ws(&base, &board), Some(&roles.owner.token)).await;
    let mut visitor = Peer::new();
    visitor.connect(&guest(&base, &token, "%20Ana%20"), None).await;
    visitor.announce(r##"{"user":{"name":"The owner","guest":false,"color":"#0af"},"cursor":{"x":1,"y":2}}"##).await;
    owner.pump(QUIET).await;
    let seen: Value = serde_json::from_str(owner.presence.last().unwrap()).unwrap();
    assert_eq!(seen["user"], json!({"name": "Ana", "guest": true, "color": "#0af"}));
    assert_eq!(seen["cursor"], json!({"x": 1, "y": 2}));
}

#[tokio::test]
async fn guest_names_and_tokens_are_checked_before_the_upgrade() {
    let Some(pool) = common::pool("toile_share_names").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let board = fixture::id(&fixture::board(&app, &roles.owner, None, "Names").await);
    let (_, token) = fixture::link(&app, &roles.owner, &board, "edit").await;
    let long = "x".repeat(41);
    for name in ["", "%20%20", "a%07b", "a%0Ab", long.as_str()] {
        let status = app.call("GET", &format!("/api/share/{token}/ws?name={name}"), None, None).await.status;
        assert_eq!(status, 400, "{name:?}");
    }
    assert_eq!(app.call("GET", "/api/share/nope/ws?name=Ana", None, None).await.status, 404);
    assert_eq!(app.call("GET", "/api/share/nope", None, None).await.status, 404);
}

#[tokio::test]
async fn revoking_a_link_closes_its_sockets_with_1008_and_kills_the_token() {
    let Some(pool) = common::pool("toile_share_revoke").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let board = fixture::id(&fixture::board(&app, &roles.owner, Some(&roles.space), "Gone").await);
    let base = app.serve().await;
    let (id, token) = fixture::link(&app, &roles.owner, &board, "edit").await;
    let (_, other) = fixture::link(&app, &roles.owner, &board, "view").await;
    let (mut revoked, mut kept) = (Peer::new(), Peer::new());
    revoked.connect(&guest(&base, &token, "Bob"), None).await;
    kept.connect(&guest(&base, &other, "Eve"), None).await;
    let path = format!("/api/boards/{board}/shares/{id}");
    assert_eq!(app.call("DELETE", &path, Some(&roles.outsider), None).await.status, 404);
    assert_eq!(app.call("DELETE", &path, Some(&roles.member), None).await.status, 403);
    assert_eq!(app.call("DELETE", &path, Some(&roles.admin), None).await.status, 204);
    assert_eq!(app.call("DELETE", &path, Some(&roles.admin), None).await.status, 404);
    revoked.pump(QUIET).await;
    kept.pump(QUIET).await;
    assert_eq!((revoked.closed, kept.closed), (Some(1008), None));
    assert_eq!(app.call("GET", &format!("/api/share/{token}"), None, None).await.status, 404);
    assert_eq!(app.call("DELETE", &format!("/api/boards/{board}"), Some(&roles.owner), None).await.status, 204);
    kept.pump(QUIET).await;
    assert_eq!(kept.closed, Some(1008));
    assert_eq!(app.call("GET", &format!("/api/share/{other}"), None, None).await.status, 404);
}

#[tokio::test]
async fn the_public_share_routes_are_rate_limited_per_client() {
    let Some(pool) = common::pool("toile_share_limit").await else { return };
    let app = common::app(pool).await;
    let mut statuses = Vec::new();
    for _ in 0..40 {
        statuses.push(app.call("GET", "/api/share/nope", None, None).await.status.as_u16());
    }
    assert!(statuses.contains(&404));
    assert_eq!(*statuses.last().unwrap(), 429, "{statuses:?}");
}
