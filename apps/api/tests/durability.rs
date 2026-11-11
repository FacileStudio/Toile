mod common;

use std::time::Duration;

use common::board::rect;
use common::server::{database, drop_database, free_port, start};
use common::ysync::Peer;
use sqlx::PgPool;

async fn session(pool: &PgPool) -> String {
    let user: i64 = sqlx::query_scalar("INSERT INTO porte_users (email) VALUES ('kill@example.test') RETURNING id")
        .fetch_one(pool)
        .await
        .unwrap();
    let token = porte::new_token().unwrap();
    sqlx::query("INSERT INTO porte_sessions (token_hash, user_id) VALUES ($1, $2)")
        .bind(porte::hash_token(&token))
        .bind(user)
        .execute(pool)
        .await
        .unwrap();
    token
}

async fn board(port: u16, token: &str) -> String {
    let answer: serde_json::Value = reqwest::Client::new()
        .post(format!("http://127.0.0.1:{port}/api/boards"))
        .bearer_auth(token)
        .json(&serde_json::json!({"name": "Durable"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    answer["id"].as_str().unwrap().to_owned()
}

async fn pending(pool: &PgPool, id: &str) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM board_updates WHERE board_id = $1::uuid")
        .bind(id)
        .fetch_one(pool)
        .await
        .unwrap()
}

#[tokio::test]
async fn a_killed_server_keeps_every_flushed_update_and_a_stopped_one_snapshots() {
    let Some((url, pool)) = database().await else { return };
    let server = start(&url, free_port()).await;
    let token = session(&pool).await;
    let id = board(server.port, &token).await;
    let _peer = draw(server.port, &id, &token, "before-kill").await;
    tokio::time::sleep(Duration::from_secs(1)).await;
    let server = start(&url, server.kill()).await;
    let survived = read(server.port, &id, &token).await;
    let mut after = draw(server.port, &id, &token, "before-stop").await;
    let took = server.terminate();
    after.pump(Duration::from_millis(500)).await;
    let (left, closed) = (pending(&pool, &id).await, after.closed);
    let server = start(&url, free_port()).await;
    let shapes = read(server.port, &id, &token).await;
    server.kill();
    pool.close().await;
    drop_database(&url).await;
    assert_eq!(survived, ["before-kill"]);
    assert_eq!((left, closed), (0, Some(1013)));
    assert!(took < Duration::from_secs(5), "the drain took {took:?}");
    assert_eq!(shapes, ["before-kill", "before-stop"]);
}

async fn read(port: u16, id: &str, token: &str) -> Vec<String> {
    let mut fresh = Peer::new();
    fresh.connect(&format!("ws://127.0.0.1:{port}/api/boards/{id}/ws"), Some(token)).await;
    fresh.shapes()
}

async fn draw(port: u16, id: &str, token: &str, shape: &str) -> Peer {
    let mut peer = Peer::new();
    peer.connect(&format!("ws://127.0.0.1:{port}/api/boards/{id}/ws"), Some(token)).await;
    peer.edit(|txn, shapes| rect(txn, shapes, shape, 1.0)).await;
    peer.pump(Duration::from_millis(100)).await;
    peer
}
