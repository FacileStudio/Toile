use std::sync::{Arc, Mutex};

use api::modules::suite::App as SuiteApp;
use api::scene::input::{Kind, NewShape};
use axum::Router;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use serde_json::{Value, json};

#[derive(Default)]
pub struct Bulle {
    pub calls: Mutex<Vec<(String, Value)>>,
    pub refuse: Mutex<Option<u16>>,
    pub nameless: Mutex<bool>,
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

pub async fn start() -> (Arc<Bulle>, api::modules::suite::Apps) {
    let fake = Arc::new(Bulle::default());
    let router = Router::new()
        .route("/api/ai/complete", post(complete))
        .route("/api/assistant", get(profile))
        .with_state(fake.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    let (_, mut apps) = super::fakes::start().await;
    apps.bulle = SuiteApp { key: "bulle", url, audience: "bulle".into() };
    (fake, apps)
}

pub async fn stickies(app: &super::App, board: &str, texts: &[&str]) -> Vec<String> {
    let id = uuid::Uuid::parse_str(board).unwrap();
    let shapes = texts.iter().map(|t| NewShape::new(Kind::Sticky, *t)).collect();
    api::scene::add(&app.state, id, shapes).await.unwrap().into_iter().map(|p| p.id).collect()
}
