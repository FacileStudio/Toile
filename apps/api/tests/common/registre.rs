use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use porte::exchange::{Config, Exchanger};
use porte::http_client::{HttpClient, HttpRequest, HttpResponse, Transport, TransportError};
use sqlx::PgPool;

use super::ISSUER;

#[derive(Default)]
pub struct Registre {
    pub exchanges: AtomicU64,
}

fn reply(status: u16, body: serde_json::Value) -> HttpResponse {
    http::Response::builder().status(status).body(body.to_string().into_bytes()).unwrap()
}

impl Registre {
    pub fn exchanger(self: &Arc<Self>, pool: &PgPool) -> Exchanger {
        let identities = porte::pg::Store::new(pool.clone()).identities();
        let mut config = Config::new(ISSUER, "toile", "secret", identities);
        config.http = HttpClient::from_transport(self.clone());
        Exchanger::new(config).unwrap()
    }

    fn token(&self, form: &HashMap<String, String>) -> HttpResponse {
        let user = form["subject_token"].trim_start_matches("at-");
        let serial = self.exchanges.fetch_add(1, Ordering::SeqCst);
        let token = format!("x.{}.{user}.{serial}", form["audience"]);
        reply(200, serde_json::json!({"access_token": token, "expires_in": 300}))
    }
}

pub fn holder(token: &str) -> &str {
    token.split('.').nth(2).unwrap_or_default()
}

#[async_trait::async_trait]
impl Transport for Registre {
    async fn send(&self, request: HttpRequest) -> Result<HttpResponse, TransportError> {
        if request.uri().path().ends_with("openid-configuration") {
            return Ok(reply(200, serde_json::json!({"token_endpoint": format!("{ISSUER}/oauth/token")})));
        }
        let form: HashMap<String, String> = url::form_urlencoded::parse(request.body()).into_owned().collect();
        Ok(match form["grant_type"].as_str() {
            "refresh_token" => {
                let user = form["refresh_token"].trim_start_matches("rt-").to_owned();
                reply(200, serde_json::json!({"access_token": format!("at-{user}"), "expires_in": 900}))
            }
            _ => self.token(&form),
        })
    }
}
