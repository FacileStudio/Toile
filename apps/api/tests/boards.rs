mod common;

use std::time::Duration;

use common::board::{rect, ws};
use common::fixture;
use common::ysync::Peer;
use serde_json::json;

#[tokio::test]
async fn a_board_renames_and_duplicates_with_its_live_content() {
    let Some(pool) = common::pool("toile_boards_copy").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let id = fixture::id(&fixture::board(&app, &roles.member, Some(&roles.space), "Plan").await);
    let path = format!("/api/boards/{id}");
    let renamed = app.call("PATCH", &path, Some(&roles.member), Some(json!({"name": "  Roadmap "}))).await.json();
    assert_eq!(renamed["name"], json!("Roadmap"));
    assert_eq!(
        app.call("PATCH", &path, Some(&roles.member), Some(json!({"title": "x"}))).await.status.as_u16() / 100,
        4
    );
    let mut peer = Peer::new();
    peer.connect(&ws(&app.serve().await, &id), Some(&roles.member.token)).await;
    peer.edit(|txn, shapes| rect(txn, shapes, "unsaved", 1.0)).await;
    peer.pump(Duration::from_millis(100)).await;
    let copy = app.call("POST", &format!("{path}/duplicate"), Some(&roles.admin), None).await;
    assert_eq!(copy.status, 201, "{}", copy.text());
    let copy = copy.json();
    assert_eq!((copy["name"].clone(), copy["space_id"].clone()), (json!("Roadmap (copie)"), json!(roles.space)));
    let mut reader = Peer::new();
    reader.connect(&ws(&app.serve().await, &fixture::id(&copy)), Some(&roles.member.token)).await;
    assert_eq!(reader.shapes(), ["unsaved"]);
    let named =
        app.call("POST", &format!("{path}/duplicate"), Some(&roles.member), Some(json!({"name": "Fork"}))).await;
    assert_eq!(named.json()["name"], json!("Fork"));
}

#[tokio::test]
async fn the_trash_lists_restores_and_purges_for_owners_only() {
    let Some(pool) = common::pool("toile_boards_trash").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let id = fixture::id(&fixture::board(&app, &roles.member, Some(&roles.space), "Old").await);
    let path = format!("/api/boards/{id}");
    let trash = format!("/api/boards?space_id={}&trash=true", roles.space);
    assert_eq!(app.call("DELETE", &path, Some(&roles.member), None).await.status, 403);
    assert_eq!(app.call("DELETE", &format!("{path}?permanent=true"), Some(&roles.owner), None).await.status, 404);
    assert_eq!(app.call("DELETE", &path, Some(&roles.owner), None).await.status, 204);
    assert_eq!(app.call("GET", &path, Some(&roles.owner), None).await.status, 404);
    assert_eq!(app.call("GET", &trash, Some(&roles.member), None).await.status, 403);
    let listed = app.call("GET", &trash, Some(&roles.admin), None).await.json();
    assert_eq!(listed[0]["id"], json!(id));
    assert!(listed[0]["deleted_at"].is_string());
    assert_eq!(app.call("POST", &format!("{path}/restore"), Some(&roles.member), None).await.status, 404);
    let restored = app.call("POST", &format!("{path}/restore"), Some(&roles.admin), None).await;
    assert_eq!(restored.status, 200);
    assert!(restored.json().get("deleted_at").is_none());
    assert_eq!(app.call("POST", &format!("{path}/restore"), Some(&roles.admin), None).await.status, 404);
    assert_eq!(app.call("DELETE", &path, Some(&roles.owner), None).await.status, 204);
    assert_eq!(app.call("DELETE", &format!("{path}?permanent=true"), Some(&roles.owner), None).await.status, 204);
    assert_eq!(app.call("GET", &trash, Some(&roles.owner), None).await.json(), json!([]));
}

#[tokio::test]
async fn the_sweep_purges_only_boards_trashed_past_retention() {
    let Some(pool) = common::pool("toile_boards_sweep").await else { return };
    let app = common::app(pool.clone()).await;
    let roles = fixture::roles(&app).await;
    let (old, new) = (
        fixture::id(&fixture::board(&app, &roles.member, None, "Old").await),
        fixture::id(&fixture::board(&app, &roles.member, None, "New").await),
    );
    let trash = "UPDATE boards SET deleted_at = now() - $2::interval WHERE id = $1::uuid";
    sqlx::query(trash).bind(&old).bind("31 days").execute(&pool).await.unwrap();
    sqlx::query(trash).bind(&new).bind("1 day").execute(&pool).await.unwrap();
    assert_eq!(api::modules::boards::trash::sweep(&app.state).await.unwrap(), 1);
    let left: Vec<String> = sqlx::query_scalar("SELECT name FROM boards WHERE owner_user_id = $1")
        .bind(roles.member.user_id)
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(left, ["New"]);
}

#[tokio::test]
async fn a_stored_thumbnail_is_served_to_viewers_with_a_versioned_url() {
    let Some(pool) = common::pool("toile_boards_thumb").await else { return };
    let app = common::app(pool.clone()).await;
    let roles = fixture::roles(&app).await;
    let id = fixture::id(&fixture::board(&app, &roles.member, Some(&roles.space), "Pic").await);
    let path = format!("/api/boards/{id}/thumbnail.png");
    let rendered = app.call("GET", &path, Some(&roles.member), None).await;
    assert!(rendered.status == 200 && rendered.bytes.starts_with(b"\x89PNG\r\n"), "{}", rendered.status);
    sqlx::query("UPDATE boards SET thumbnail = $2, thumbnail_at = now() WHERE id = $1::uuid")
        .bind(&id)
        .bind(b"\x89PNG".to_vec())
        .execute(&pool)
        .await
        .unwrap();
    let png = app.call("GET", &path, Some(&roles.owner), None).await;
    assert_eq!((png.status.as_u16(), png.headers["content-type"].to_str().unwrap()), (200, "image/png"));
    assert_eq!(app.call("GET", &path, Some(&roles.outsider), None).await.status, 404);
    let board = app.call("GET", &format!("/api/boards/{id}"), Some(&roles.member), None).await.json();
    assert!(board["thumbnail_url"].as_str().unwrap().starts_with(&format!("/api/boards/{id}/thumbnail.png?v=")));
}
