mod common;

use std::sync::{Arc, Mutex};

use api::auth::SpacesApply;
use api::modules::suite::App as SuiteApp;
use api::scene::input::{Kind, NewShape};
use axum::Router;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use common::directory::{base, snapshot, space};
use common::fixture;
use porte::directory::Person;
use serde_json::{Value, json};

#[derive(Default)]
struct Bulle {
    calls: Mutex<Vec<(String, Value)>>,
    refuse: Mutex<Option<u16>>,
    nameless: Mutex<bool>,
}

fn refusal(status: u16) -> Response {
    let body = match status {
        402 => {
            json!({"error": {"code": "payment_required", "message": "Upgrade", "upgrade_url": "https://sso.test/spaces"}})
        }
        _ => json!({"error": {"code": "rate_limited", "message": "at most 200 completions per day"}}),
    };
    (StatusCode::from_u16(status).unwrap(), axum::Json(body)).into_response()
}

fn ids(input: &str) -> Vec<String> {
    input
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok()?["id"].as_str().map(str::to_owned))
        .collect()
}

fn drawing(schema: &Value, input: &str) -> Value {
    let required = schema["required"].to_string();
    if required.contains("groups") {
        let ids = ids(input);
        let (a, b) = ids.split_at(ids.len() / 2);
        let b: Vec<String> = b.iter().cloned().chain(["ghost".to_owned()]).collect();
        return json!({"groups": [{"title": "Prix", "shapes": a}, {"title": "Délais", "shapes": b}]});
    }
    if required.contains("branches") {
        return json!({"title": "Lancement", "branches": [{"text": "Site", "children": ["Texte", "Visuels"]}, {"text": "Presse", "children": []}]});
    }
    if required.contains("nodes") {
        return json!({"nodes": [{"id": "a", "text": "Début", "kind": "start"}, {"id": "b", "text": "Fin", "kind": "end"}], "edges": [{"from": "a", "to": "b"}]});
    }
    json!({"title": "Idées", "stickies": ["Un", "Deux", "Trois"]})
}

async fn complete(State(fake): State<Arc<Bulle>>, headers: HeaderMap, axum::Json(body): axum::Json<Value>) -> Response {
    let token = headers.get(header::AUTHORIZATION).and_then(|v| v.to_str().ok()).unwrap_or_default();
    let parts: Vec<&str> = token.trim_start_matches("Bearer ").split('.').collect();
    if parts.len() != 4 || parts[1] != "bulle" {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    fake.calls.lock().unwrap().push((parts[2].to_owned(), body.clone()));
    if let Some(status) = *fake.refuse.lock().unwrap() {
        return refusal(status);
    }
    let input = body["input"].as_str().unwrap_or_default();
    match body.get("schema") {
        Some(schema) => axum::Json(json!({"json": drawing(schema, input), "model": "fake"})).into_response(),
        None => axum::Json(json!({"text": "## Résumé\n- deux idées\n", "model": "fake"})).into_response(),
    }
}

async fn profile(State(fake): State<Arc<Bulle>>, headers: HeaderMap) -> Response {
    let token = headers.get(header::AUTHORIZATION).and_then(|v| v.to_str().ok()).unwrap_or_default();
    match (token.split('.').nth(1), *fake.nameless.lock().unwrap()) {
        (Some("bulle"), false) => axum::Json(json!({"name": "  Pixel ", "archetype": "calm"})).into_response(),
        (Some("bulle"), true) => StatusCode::NOT_FOUND.into_response(),
        _ => StatusCode::UNAUTHORIZED.into_response(),
    }
}

async fn start() -> (Arc<Bulle>, api::modules::suite::Apps) {
    let fake = Arc::new(Bulle::default());
    let router = Router::new()
        .route("/api/ai/complete", post(complete))
        .route("/api/assistant", get(profile))
        .with_state(fake.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    let (_, mut apps) = common::fakes::start().await;
    apps.bulle = SuiteApp { key: "bulle", url, audience: "bulle".into() };
    (fake, apps)
}

async fn stickies(app: &common::App, board: &str, texts: &[&str]) -> Vec<String> {
    let id = uuid::Uuid::parse_str(board).unwrap();
    let shapes = texts.iter().map(|t| NewShape::new(Kind::Sticky, *t)).collect();
    api::scene::add(&app.state, id, shapes).await.unwrap().into_iter().map(|p| p.id).collect()
}

#[tokio::test]
async fn an_editor_summarizes_clusters_and_generates_through_bulle() {
    let Some(pool) = common::pool("toile_ai").await else { return };
    let (bulle, apps) = start().await;
    let app = common::app_with(pool, apps).await;
    let roles = fixture::roles(&app).await;
    let board = fixture::id(&fixture::board(&app, &roles.owner, Some(&roles.space), "Atelier").await);
    let notes = stickies(&app, &board, &["Trop cher", "Livraison lente", "Prix flou", "Retard"]).await;

    let summary = app.call("POST", &format!("/api/boards/{board}/ai/summarize"), Some(&roles.member), None).await;
    assert_eq!(summary.status, 200, "{}", summary.text());
    assert_eq!(summary.json()["markdown"], "## Résumé\n- deux idées");
    let (user, sent) = bulle.calls.lock().unwrap()[0].clone();
    assert_eq!(user, roles.member.user_id.to_string());
    assert!(sent["input"].as_str().unwrap().contains("\"Trop cher\"") && sent.get("schema").is_none());

    let mut selection = notes.clone();
    selection.push("notasticky".into());
    let path = format!("/api/boards/{board}/ai/cluster");
    let clustered = app.call("POST", &path, Some(&roles.member), Some(json!({"shapes": selection}))).await;
    assert_eq!(clustered.status, 200, "{}", clustered.text());
    let frames = clustered.json()["frames"].as_array().unwrap().clone();
    assert_eq!(
        (frames.len(), frames[0]["title"].clone(), frames[1]["shapes"].as_array().unwrap().len()),
        (2, json!("Prix"), 2)
    );
    let live = api::scene::load(&app.state, uuid::Uuid::parse_str(&board).unwrap()).await.unwrap();
    for frame in &frames {
        let id = frame["frame"].as_str().unwrap();
        let frame_z = &live.iter().find(|s| s.id == id).unwrap().z;
        for sticky in frame["shapes"].as_array().unwrap() {
            let shape = live.iter().find(|s| s.id == sticky.as_str().unwrap()).unwrap();
            assert_eq!(shape.parent.as_deref(), Some(id));
            assert!(&shape.z > frame_z, "stickies paint above their frame");
        }
    }

    generate(&app, &roles, &board).await;
}

async fn generate(app: &common::App, roles: &fixture::Roles, board: &str) {
    let path = format!("/api/boards/{board}/ai/generate");
    for (kind, count) in [("mindmap", 9), ("flowchart", 3), ("stickies", 4)] {
        let body = json!({"prompt": "Le lancement de la rentrée", "kind": kind});
        let answer = app.call("POST", &path, Some(&roles.member), Some(body)).await;
        assert_eq!(answer.status, 200, "{kind}: {}", answer.text());
        assert_eq!(answer.json()["created"].as_array().unwrap().len(), count, "{kind}");
    }
    let bad = app.call("POST", &path, Some(&roles.member), Some(json!({"prompt": "x", "kind": "poem"}))).await;
    assert_eq!(bad.status, 400);
    let empty = app.call("POST", &path, Some(&roles.member), Some(json!({"prompt": "  ", "kind": "stickies"}))).await;
    assert_eq!(empty.status, 400);
}

#[tokio::test]
async fn no_ai_access_answers_402_before_bulle_and_bulle_refusals_pass_through() {
    let Some(pool) = common::pool("toile_ai_gate").await else { return };
    let (bulle, apps) = start().await;
    let app = common::app_with(pool.clone(), apps).await;
    let roles = fixture::roles(&app).await;
    let ada = app.person("ada").await;
    let personal = fixture::id(&fixture::board(&app, &ada, None, "Perso").await);
    let shared = fixture::id(&fixture::board(&app, &roles.owner, Some(&roles.space), "Partagé").await);
    stickies(&app, &personal, &["Une idée"]).await;
    stickies(&app, &shared, &["Une idée"]).await;
    let summarize = format!("/api/boards/{shared}/ai/summarize");
    assert_eq!(app.call("POST", &summarize, None, None).await.status, 401);
    assert_eq!(app.call("POST", &summarize, Some(&roles.outsider), None).await.status, 404);
    assert!(bulle.calls.lock().unwrap().is_empty());

    *bulle.refuse.lock().unwrap() = Some(402);
    let passed = app.call("POST", &summarize, Some(&roles.member), None).await;
    assert_eq!((passed.status.as_u16(), passed.json()["error"]["key"].clone()), (402, json!("ai.access")));
    *bulle.refuse.lock().unwrap() = Some(429);
    assert_eq!(app.call("POST", &summarize, Some(&roles.member), None).await.status, 429);
    *bulle.refuse.lock().unwrap() = None;
    bulle.calls.lock().unwrap().clear();

    let mut snap = snapshot(base(), vec![space("unused", &[])]);
    snap.entitlements_revision = 1;
    snap.people =
        vec![Person { subject: ada.subject.clone(), entitlements: [("ai.access".into(), json!(false))].into() }];
    let apply = SpacesApply::new(pool, app.state.hub.clone(), app.state.suite.plans.clone());
    porte::directory::Apply::apply(&apply, &snap).await.unwrap();

    let refused = app.call("POST", &format!("/api/boards/{personal}/ai/summarize"), Some(&ada), None).await;
    assert_eq!(refused.status, 402, "{}", refused.text());
    let error = &refused.json()["error"];
    assert_eq!((error["code"].clone(), error["key"].clone()), (json!("payment_required"), json!("ai.access")));
    assert_eq!(error["upgrade_url"], "https://sso.test/spaces");
    let generate = json!({"prompt": "x", "kind": "stickies"});
    let path = format!("/api/boards/{personal}/ai/generate");
    assert_eq!(app.call("POST", &path, Some(&ada), Some(generate)).await.status, 402);
    assert!(bulle.calls.lock().unwrap().is_empty(), "a refused plan never reaches Bulle");
}

#[tokio::test]
async fn an_editor_reads_the_assistant_name_without_ai_access() {
    let Some(pool) = common::pool("toile_ai_assistant").await else { return };
    let (bulle, apps) = start().await;
    let app = common::app_with(pool.clone(), apps).await;
    let roles = fixture::roles(&app).await;
    let ada = app.person("ada").await;
    let shared = fixture::id(&fixture::board(&app, &roles.owner, Some(&roles.space), "Atelier").await);
    let personal = fixture::id(&fixture::board(&app, &ada, None, "Perso").await);
    let path = format!("/api/boards/{shared}/ai/assistant");
    let named = app.call("GET", &path, Some(&roles.member), None).await;
    assert_eq!((named.status.as_u16(), named.json()), (200, json!({"name": "Pixel"})), "{}", named.text());
    assert_eq!(app.call("GET", &path, Some(&roles.outsider), None).await.status, 404);
    *bulle.nameless.lock().unwrap() = true;
    assert_eq!(app.call("GET", &path, Some(&roles.member), None).await.json()["name"], "Bulle");
    *bulle.nameless.lock().unwrap() = false;

    let mut snap = snapshot(base(), vec![space("unused", &[])]);
    snap.entitlements_revision = 1;
    snap.people =
        vec![Person { subject: ada.subject.clone(), entitlements: [("ai.access".into(), json!(false))].into() }];
    let apply = SpacesApply::new(pool, app.state.hub.clone(), app.state.suite.plans.clone());
    porte::directory::Apply::apply(&apply, &snap).await.unwrap();
    let mine = app.call("GET", &format!("/api/boards/{personal}/ai/assistant"), Some(&ada), None).await;
    assert_eq!((mine.status.as_u16(), mine.json()["name"].clone()), (200, json!("Pixel")), "{}", mine.text());
    let summary = app.call("POST", &format!("/api/boards/{personal}/ai/summarize"), Some(&ada), None).await;
    assert_eq!(summary.status, 402, "the plan really lacks ai.access");
}
