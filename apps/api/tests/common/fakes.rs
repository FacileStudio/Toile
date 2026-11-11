use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use api::modules::suite::{App, Apps};
use axum::Router;
use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use serde_json::{Value, json};

type Objects = HashMap<(String, String), (String, Vec<u8>)>;

#[derive(Default)]
pub struct Suite {
    pub objects: Mutex<Objects>,
    pub tasks: Mutex<Vec<(String, Value)>>,
    pub hits: Mutex<HashMap<String, usize>>,
}

type Shared = Arc<Suite>;

fn caller(headers: &HeaderMap, audience: &str) -> Option<String> {
    let token = headers.get(header::AUTHORIZATION)?.to_str().ok()?.strip_prefix("Bearer ")?;
    let parts: Vec<&str> = token.split('.').collect();
    (parts.len() == 4 && parts[1] == audience).then(|| parts[2].to_owned())
}

async fn put(
    State(s): State<Shared>,
    headers: HeaderMap,
    Query(q): Query<HashMap<String, String>>,
    body: Bytes,
) -> Response {
    let Some(user) = caller(&headers, "nuage-id") else { return StatusCode::UNAUTHORIZED.into_response() };
    let mime = headers.get(header::CONTENT_TYPE).and_then(|v| v.to_str().ok()).unwrap_or_default().to_owned();
    s.objects.lock().unwrap().insert((user, q["key"].clone()), (mime, body.to_vec()));
    (StatusCode::CREATED, axum::Json(json!({"key": q["key"], "name": q["name"]}))).into_response()
}

async fn object(State(s): State<Shared>, headers: HeaderMap, Query(q): Query<HashMap<String, String>>) -> Response {
    let Some(user) = caller(&headers, "nuage-id") else { return StatusCode::UNAUTHORIZED.into_response() };
    match s.objects.lock().unwrap().get(&(user, q["key"].clone())) {
        Some((mime, bytes)) => ([(header::CONTENT_TYPE, mime.clone())], bytes.clone()).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn task(State(s): State<Shared>, headers: HeaderMap, Path(project): Path<String>, body: Bytes) -> Response {
    let Some(user) = caller(&headers, "opus-id") else { return StatusCode::UNAUTHORIZED.into_response() };
    if project != "7" {
        return StatusCode::NOT_FOUND.into_response();
    }
    let body: Value = serde_json::from_slice(&body).unwrap();
    let title = body["title"].clone();
    s.tasks.lock().unwrap().push((user, body));
    let answer = json!({"id": 42, "workspaceId": 3, "projectId": 7, "title": title, "status": "todo"});
    (StatusCode::CREATED, axum::Json(answer)).into_response()
}

async fn document(State(s): State<Shared>, headers: HeaderMap, Path(id): Path<String>) -> Response {
    item(&s, &headers, "documents", &id)
}

async fn opus_task(State(s): State<Shared>, headers: HeaderMap, Path(id): Path<String>) -> Response {
    item(&s, &headers, "task", &id)
}

fn item(s: &Suite, headers: &HeaderMap, kind: &str, id: &str) -> Response {
    *s.hits.lock().unwrap().entry(kind.to_owned()).or_default() += 1;
    let audience = if kind == "task" { "opus-id" } else { "docs-id" };
    if caller(headers, audience).is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    match (kind, id) {
        ("documents", "6f1e2d3c-4b5a-4968-8776-655443322110") => {
            axum::Json(json!({"title": "Plan de rentrée"})).into_response()
        }
        ("task", "42") => axum::Json(json!({"title": "Ship it", "status": "in_progress"})).into_response(),
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}

/// One fake for Nuage, Opus, Docs, Feuilles and Agenda, each app on its own port.
pub async fn start() -> (Shared, Apps) {
    let shared = Shared::default();
    let router = Router::new()
        .route("/api/app/object", get(object).put(put))
        .route("/api/task/{id}", post(task).get(opus_task))
        .route("/api/documents/{id}", get(document))
        .with_state(shared.clone());
    let mut urls = Vec::new();
    for _ in 0..5 {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        urls.push(format!("http://{}", listener.local_addr().unwrap()));
        let router = router.clone();
        tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    }
    let app = |key: &'static str, url: &String| App { key, url: url.clone(), audience: format!("{key}-id") };
    let apps = Apps {
        nuage: app("nuage", &urls[0]),
        opus: app("opus", &urls[1]),
        docs: app("docs", &urls[2]),
        feuilles: app("feuilles", &urls[3]),
        agenda: app("agenda", &urls[4]),
        bulle: App::default(),
    };
    (shared, apps)
}
