mod common;

use common::fixture;
use serde_json::json;

#[tokio::test]
async fn a_created_board_is_listed_newest_first_in_its_scope_only() {
    let Some(pool) = common::pool("toile_life").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let first = fixture::board(&app, &roles.member, Some(&roles.space), "  Rétro  ").await;
    assert_eq!(first["name"], json!("Rétro"));
    assert_eq!(first["space_id"], json!(roles.space));
    assert_eq!(first["access"], json!("editor"));
    let second = fixture::board(&app, &roles.owner, Some(&roles.space), "").await;
    assert_eq!(second["name"], json!("Sans titre"));
    fixture::board(&app, &roles.member, None, "À moi").await;
    let path = format!("/api/boards?space_id={}", roles.space);
    let listed = app.call("GET", &path, Some(&roles.member), None).await.json();
    let ids: Vec<&str> = listed.as_array().unwrap().iter().map(|b| b["id"].as_str().unwrap()).collect();
    assert_eq!(ids, [fixture::id(&second), fixture::id(&first)]);
    let personal = app.call("GET", "/api/boards", Some(&roles.member), None).await.json();
    assert_eq!(personal.as_array().unwrap().len(), 1);
    assert_eq!(personal[0]["space_id"], json!(null));
}

#[tokio::test]
async fn a_deleted_board_disappears() {
    let Some(pool) = common::pool("toile_deleted").await else { return };
    let app = common::app(pool.clone()).await;
    let roles = fixture::roles(&app).await;
    let board = fixture::id(&fixture::board(&app, &roles.owner, Some(&roles.space), "Ancien").await);
    sqlx::query("UPDATE boards SET deleted_at = now() WHERE id = $1::uuid").bind(&board).execute(&pool).await.unwrap();
    assert_eq!(app.call("GET", &format!("/api/boards/{board}"), Some(&roles.owner), None).await.status, 404);
    let path = format!("/api/boards?space_id={}", roles.space);
    assert_eq!(app.call("GET", &path, Some(&roles.owner), None).await.json(), json!([]));
}

#[tokio::test]
async fn bad_create_bodies_are_refused() {
    let Some(pool) = common::pool("toile_bodies").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let long = json!({"name": "x".repeat(201)});
    assert_eq!(app.call("POST", "/api/boards", Some(&roles.owner), Some(long)).await.status, 400);
    let unknown = json!({"name": "x", "colour": "red"});
    assert_eq!(app.call("POST", "/api/boards", Some(&roles.owner), Some(unknown)).await.status.as_u16() / 100, 4);
}

#[tokio::test]
async fn the_probes_version_and_reference_answer() {
    let Some(pool) = common::pool("toile_meta").await else { return };
    let app = common::app(pool).await;
    let missing = tronc::apiref::undocumented(&app.router, &api::apiref::document(), &api::apiref::config(), &[]);
    assert!(missing.is_empty(), "routes missing from the API document: {missing:?}");
    assert_eq!(app.call("GET", "/docs/openapi.json", None, None).await.json()["info"]["title"], json!("Toile API"));
    assert_eq!(app.call("GET", "/version", None, None).await.json()["board_schema"], json!(1));
    assert_eq!(app.call("GET", "/api/health", None, None).await.status, 200);
    assert_eq!(app.call("GET", "/api/auth/session", None, None).await.json(), json!({"user": null}));
    assert_eq!(app.call("GET", "/api/no-such-route", None, None).await.status, 404);
}
