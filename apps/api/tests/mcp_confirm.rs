mod common;

use std::sync::Arc;

use axum::body::Body;
use common::http::{browser_cookie, raw, with_cookie};
use common::mcp::{Served, args, served, shapes, sticky};
use serde_json::{Value, json};
use tronc::mcptest::{self, ClientOptions, ConfirmCase, Target, call, connect, decode};

async fn target(state: api::state::AppState, router: axum::Router, token: String) -> Target {
    let made = common::send(&router, "POST", "/api/boards", Some(&token), Some(json!({"name": "T"}))).await;
    let id = made.json()["id"].as_str().unwrap().to_owned();
    let shape = sticky(&state, &id).await;
    let args = args(json!({"board_id": id, "ids": [shape]}));
    let runs = move || -> mcptest::Fut<usize> {
        let (state, id) = (state.clone(), id.clone());
        Box::pin(async move { usize::from(shapes(&state, &id).await.is_empty()) })
    };
    Target { args, runs: Arc::new(runs) }
}

fn delete_case(served: &Served) -> ConfirmCase {
    let (state, router, token) = (served.app.state.clone(), served.app.router.clone(), served.person.token.clone());
    let setup = move || -> mcptest::Fut<Target> { Box::pin(target(state.clone(), router.clone(), token.clone())) };
    ConfirmCase { tool: "toile_delete_shapes".into(), setup: Arc::new(setup) }
}

#[tokio::test]
async fn the_mcp_endpoint_passes_tronc_conformance_with_deletes_confirmed() {
    let Some(served) = served("toile_mcp_kit").await else { return };
    let user = served.person.user_id;
    let token = Arc::new(move |scopes: &[&str]| {
        let scopes = if scopes.is_empty() { "toile.read,toile.write".to_owned() } else { scopes.join(",") };
        format!("{user}|{scopes}")
    });
    let confirm = vec![delete_case(&served)];
    mcptest::run(&mcptest::Config { url: served.url.clone(), token, confirm }).await;
}

#[tokio::test]
async fn a_delete_waits_for_the_person_on_the_pending_page() {
    let Some(served) = served("toile_mcp_delete").await else { return };
    let id = served.board("Tidy").await;
    let shape = sticky(&served.app.state, &id).await;
    let client = connect(&served.url, &served.token(), ClientOptions::default()).await;
    let pending: Value =
        decode(&call(&client, "toile_delete_shapes", args(json!({"board_id": id, "ids": [shape]}))).await);
    assert_eq!(pending["status"], json!("pending_confirmation"));
    assert_eq!(shapes(&served.app.state, &id).await, std::slice::from_ref(&shape));
    let action = pending["confirm_url"].as_str().unwrap().rsplit('=').next().unwrap().to_owned();
    let cookie = browser_cookie(&served.app.kits.sessions, served.person.user_id).await;
    let list = with_cookie(&served.app.router, "GET", "/api/mcp/pending", &cookie).await.json();
    assert_eq!(list["actions"][0]["id"], json!(action));
    assert_eq!(list["actions"][0]["state"], json!("ready"));
    assert!(list["actions"][0]["target"].as_str().unwrap().contains(&id));
    let path = format!("/api/mcp/pending/{action}/confirm");
    let done = with_cookie(&served.app.router, "POST", &path, &cookie).await;
    assert_eq!(done.json(), json!({"status": "done"}), "{}", done.text());
    assert!(shapes(&served.app.state, &id).await.is_empty());
}

async fn plain_session(served: &Served, token: String) -> String {
    sqlx::query(
        "INSERT INTO porte_sessions (token_hash, user_id, expires_at) VALUES ($1, $2, now() + interval '1 hour')",
    )
    .bind(porte::hash_token(&token))
    .bind(served.person.user_id)
    .execute(&served.app.pool)
    .await
    .unwrap();
    token
}

#[tokio::test]
async fn only_a_browser_login_confirms_never_a_cli_api_or_old_cookie() {
    let Some(served) = served("toile_mcp_own").await else { return };
    let id = served.board("Guarded").await;
    let shape = sticky(&served.app.state, &id).await;
    let client = connect(&served.url, &served.token(), ClientOptions::default()).await;
    let pending: Value =
        decode(&call(&client, "toile_delete_shapes", args(json!({"board_id": id, "ids": [shape]}))).await);
    let action = pending["confirm_url"].as_str().unwrap().rsplit('=').next().unwrap().to_owned();
    let confirm = format!("/api/mcp/pending/{action}/confirm");
    let browser = browser_cookie(&served.app.kits.sessions, served.person.user_id).await;
    let name = browser.split_once('=').unwrap().0;
    let (cli, _) = served.app.kits.sessions.issue(served.person.user_id, "").await.unwrap();
    let refused = [
        served.person.token.clone(),
        cli,
        plain_session(&served, format!("b-{}", porte::new_token().unwrap())).await,
        plain_session(&served, porte::new_token().unwrap()).await,
    ];
    for token in &refused {
        let cookie = &format!("{name}={token}");
        let answer = with_cookie(&served.app.router, "POST", &confirm, cookie).await;
        assert_eq!(answer.status, 403, "{cookie}: {}", answer.text());
        assert_eq!(answer.json()["error"]["message"], json!(tronc::mcp::DELEGATED));
        let listed = with_cookie(&served.app.router, "GET", "/api/mcp/pending", cookie).await;
        assert_eq!(listed.status, 403, "{cookie}");
    }
    assert_eq!(served.app.call("POST", &confirm, Some(&served.person), None).await.status, 403);
    assert_eq!(shapes(&served.app.state, &id).await, std::slice::from_ref(&shape));
    let done = with_cookie(&served.app.router, "POST", &confirm, &browser).await;
    assert_eq!(done.json(), json!({"status": "done"}), "{}", done.text());
    assert!(shapes(&served.app.state, &id).await.is_empty());
}

#[tokio::test]
async fn a_cookie_confirm_without_the_csrf_header_is_refused() {
    let Some(served) = served("toile_mcp_csrf").await else { return };
    let id = served.board("Forged").await;
    let shape = sticky(&served.app.state, &id).await;
    let client = connect(&served.url, &served.token(), ClientOptions::default()).await;
    let pending: Value =
        decode(&call(&client, "toile_delete_shapes", args(json!({"board_id": id, "ids": [shape]}))).await);
    let action = pending["confirm_url"].as_str().unwrap().rsplit('=').next().unwrap().to_owned();
    let browser = browser_cookie(&served.app.kits.sessions, served.person.user_id).await;
    let forged = http::Request::post(format!("/api/mcp/pending/{action}/confirm"))
        .header(http::header::HOST, "toile.test")
        .header(http::header::COOKIE, &browser)
        .body(Body::empty())
        .unwrap();
    let forged = raw(&served.app.router, forged).await;
    assert_eq!(forged.status, 403, "{}", forged.text());
    assert!(forged.text().contains(porte::CSRF_HEADER_NAME), "{}", forged.text());
    assert_eq!(shapes(&served.app.state, &id).await, std::slice::from_ref(&shape));
}
