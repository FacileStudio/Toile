mod common;

use std::sync::Arc;

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
    let list = served.app.call("GET", "/api/mcp/actions", Some(&served.person), None).await.json();
    assert_eq!(list[0]["id"], json!(action));
    let done = served.app.call("POST", &format!("/api/mcp/actions/{action}/confirm"), Some(&served.person), None).await;
    assert_eq!(done.json(), json!({"status": "done"}), "{}", done.text());
    assert!(shapes(&served.app.state, &id).await.is_empty());
}
