use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use http::request::Parts;
use serde_json::Value;
use tronc::mcp::{Options, TokenInfo, Verifier, VerifyError, VerifyFuture};

use super::{App, Person};

struct Tokens;

impl Verifier for Tokens {
    fn verify<'a>(&'a self, token: &'a str, _request: &'a Parts) -> VerifyFuture<'a> {
        Box::pin(async move {
            let (user, scopes) = token.split_once('|').ok_or(VerifyError::Invalid)?;
            Ok(TokenInfo {
                user_id: user.to_owned(),
                scopes: scopes.split(',').map(str::to_owned).collect(),
                expires_at: Some(SystemTime::now() + Duration::from_secs(600)),
                extra: HashMap::new(),
            })
        })
    }
}

pub struct Served {
    pub app: App,
    pub url: String,
    pub person: Person,
}

impl Served {
    pub fn token(&self) -> String {
        format!("{}|toile.read,toile.write", self.person.user_id)
    }
}

pub async fn served(prefix: &str) -> Option<Served> {
    let pool = super::pool(prefix).await?;
    let mut app = super::app(pool).await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/mcp", listener.local_addr().unwrap());
    app.state.confirmations = Some(api::mcp::confirmations(&app.state, &url, b"test-state-key").unwrap());
    let options = Options::new("toile", url.clone(), "https://sso.test", Arc::new(Tokens));
    app.router = api::router::build_router(app.state.clone(), &app.kits, Some(options)).unwrap();
    let router = app.router.clone().into_make_service_with_connect_info::<SocketAddr>();
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    let person = app.person("agent-owner").await;
    Some(Served { app, url, person })
}

pub fn args(value: Value) -> tronc::mcp::rmcp::model::JsonObject {
    serde_json::from_value(value).unwrap()
}

pub async fn refused(client: &tronc::mcptest::Client, tool: &str, input: Value) -> String {
    let result = tronc::mcptest::call(client, tool, args(input)).await;
    assert_eq!(result.is_error, Some(true), "{tool} should fail");
    tronc::mcptest::text(&result)
}

impl Served {
    pub async fn board(&self, name: &str) -> String {
        let answer =
            self.app.call("POST", "/api/boards", Some(&self.person), Some(serde_json::json!({"name": name}))).await;
        answer.json()["id"].as_str().unwrap().to_owned()
    }
}

pub async fn shapes(state: &api::state::AppState, id: &str) -> Vec<String> {
    let all = api::scene::load(state, uuid::Uuid::parse_str(id).unwrap()).await.unwrap();
    all.into_iter().map(|s| s.id).collect()
}

pub async fn sticky(state: &api::state::AppState, id: &str) -> String {
    let new = api::scene::input::NewShape::new(api::scene::input::Kind::Sticky, "doomed");
    let placed = api::scene::add(state, uuid::Uuid::parse_str(id).unwrap(), vec![new]).await.unwrap();
    placed[0].id.clone()
}
