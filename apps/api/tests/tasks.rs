mod common;

use common::fixture;
use serde_json::json;
use yrs::{Any, Map, MapPrelim, Out, TextPrelim, Transact};

async fn sticky(app: &common::App, board: &str, text: &str) {
    let id = uuid::Uuid::parse_str(board).unwrap();
    let text = text.to_owned();
    let edit: api::modules::rooms::Edit = Box::new(move |txn, shapes| {
        let fields: Vec<(&str, Any)> = vec![
            ("type", "sticky".into()),
            ("x", 10.0.into()),
            ("y", 20.0.into()),
            ("w", 200.0.into()),
            ("z", "a0".into()),
        ];
        let shape = shapes.insert(txn, "stickyAAAAAAAAAA", fields.into_iter().collect::<MapPrelim>());
        shape.insert(txn, "text", TextPrelim::new(text));
        let rect: Vec<(&str, Any)> = vec![("type", "rect".into()), ("x", 0.0.into()), ("y", 0.0.into())];
        shapes.insert(txn, "rectBBBBBBBBBBBB", rect.into_iter().collect::<MapPrelim>());
        Ok(())
    });
    app.state.hub.write(&app.state, id, edit).await.unwrap();
}

fn card(bytes: &[u8], id: &str) -> Vec<(String, String)> {
    let doc = api::modules::board::from_state(bytes, []).unwrap();
    let shapes = doc.get_or_insert_map("shapes");
    let txn = doc.transact();
    let Some(Out::YMap(shape)) = shapes.get(&txn, id) else { panic!("no card {id}") };
    let mut fields: Vec<(String, String)> =
        shape.iter(&txn).map(|(key, value)| (key.to_owned(), value.to_string(&txn))).collect();
    fields.sort();
    fields
}

#[tokio::test]
async fn a_sticky_becomes_an_opus_task_with_a_live_card_beside_it() {
    let Some(pool) = common::pool("toile_tasks").await else { return };
    let (opus, apps) = common::fakes::start().await;
    let opus_url = apps.opus.url.clone();
    let app = common::app_with(pool, apps).await;
    let roles = fixture::roles(&app).await;
    let board = fixture::id(&fixture::board(&app, &roles.owner, Some(&roles.space), "Rétro").await);
    sticky(&app, &board, "  \nRelancer le fournisseur\nAvant vendredi").await;
    let path = format!("/api/boards/{board}/shapes/stickyAAAAAAAAAA/task");

    let answer = app.call("POST", &path, Some(&roles.member), Some(json!({"project": 7}))).await;
    assert_eq!(answer.status, 201, "{}", answer.text());
    let body = answer.json();
    let url = format!("{opus_url}/dashboard/workspace/3/project/7/task/42");
    assert_eq!(body["task"], json!({"id": 42, "title": "Relancer le fournisseur", "url": url}));
    let sent = opus.tasks.lock().unwrap()[0].clone();
    assert_eq!(sent.0, roles.member.user_id.to_string());
    assert_eq!(sent.1["description"], "Relancer le fournisseur\nAvant vendredi");

    let live = app.state.hub.read(&app.state, uuid::Uuid::parse_str(&board).unwrap()).await.unwrap();
    let fields = card(&live, body["card"].as_str().unwrap());
    let get = |key: &str| fields.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone()).unwrap_or_default();
    assert_eq!(
        (get("type"), get("kind"), get("url"), get("x"), get("z")),
        ("card".into(), "opus".into(), url, "234".into(), "a0V".into())
    );

    refusals(&app, &roles, &board, &path).await;
}

async fn refusals(app: &common::App, roles: &fixture::Roles, board: &str, path: &str) {
    let missing = app.call("POST", path, Some(&roles.member), Some(json!({}))).await;
    assert_eq!(missing.status, 400);
    let rect = format!("/api/boards/{board}/shapes/rectBBBBBBBBBBBB/task");
    assert_eq!(app.call("POST", &rect, Some(&roles.member), Some(json!({"project": 7}))).await.status, 400);
    let gone = format!("/api/boards/{board}/shapes/nope/task");
    assert_eq!(app.call("POST", &gone, Some(&roles.member), Some(json!({"project": 7}))).await.status, 404);
    assert_eq!(app.call("POST", path, Some(&roles.outsider), Some(json!({"project": 7}))).await.status, 404);
    assert_eq!(app.call("POST", path, Some(&roles.member), Some(json!({"project": 8}))).await.status, 404);
}
