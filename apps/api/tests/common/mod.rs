#![allow(dead_code)]

pub mod board;
pub mod directory;
pub mod fakes;
pub mod fixture;
pub mod http;
pub mod mcp;
pub mod registre;
pub mod server;
pub mod upload;
pub mod ysync;

use std::sync::Arc;

use api::auth::Kits;
use api::env::Config;
use api::state::AppState;
use axum::Router;
use sqlx::PgPool;
use tronc::env::Vars;

pub use http::{Answer, send};

pub const ISSUER: &str = "https://sso.test";

pub struct App {
    pub router: Router,
    pub state: AppState,
    pub kits: Kits,
    pub pool: PgPool,
    pub registre: Arc<registre::Registre>,
}

pub struct Person {
    pub user_id: i64,
    pub subject: String,
    pub token: String,
}

pub async fn pool(prefix: &str) -> Option<PgPool> {
    let pool = tronc::testdb::pool(prefix, &api::MIGRATOR).await;
    if pool.is_none() {
        eprintln!("{}", tronc::testdb::skip_reason("docker run postgres:16-alpine on 127.0.0.1:5657"));
    }
    pool
}

pub async fn app(pool: PgPool) -> App {
    app_with(pool, api::modules::suite::Apps::default()).await
}

pub async fn app_with(pool: PgPool, apps: api::modules::suite::Apps) -> App {
    app_from(pool, &[], apps).await
}

pub async fn app_from(pool: PgPool, vars: &[(&str, &str)], apps: api::modules::suite::Apps) -> App {
    let mut pairs = vec![("DATABASE_URL", "postgres://unused/x")];
    pairs.extend_from_slice(vars);
    let config = Config::from_vars(&Vars::from_pairs(pairs)).unwrap();
    let hub = api::modules::rooms::Hub::default();
    let mut kits = Kits::new(&config, &pool, &hub).await.unwrap();
    let registre = Arc::new(registre::Registre::default());
    kits.suite.exchanger = Some(registre.exchanger(&pool));
    kits.suite.bulle = apps
        .bulle
        .enabled()
        .then(|| tronc::bulle::Client::with_timeout(&apps.bulle.url, api::modules::ai::TIMEOUT).unwrap());
    kits.suite.apps = apps;
    let state = AppState {
        pool: pool.clone(),
        sessions: kits.sessions.clone(),
        issuer: Arc::from(ISSUER),
        hub,
        socket_recheck: std::time::Duration::from_millis(200),
        suite: kits.suite.clone(),
        confirmations: None,
        sealer: None,
        public_url: Arc::from("https://toile.test"),
    };
    let router = api::router::build_router(state.clone(), &kits, None, false).unwrap();
    App { router, state, kits, pool, registre }
}

impl App {
    pub async fn serve(&self) -> String {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let router = self.router.clone().into_make_service_with_connect_info::<std::net::SocketAddr>();
        tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
        format!("ws://{addr}")
    }

    pub async fn person(&self, name: &str) -> Person {
        let subject = format!("{name}-{}", uuid::Uuid::new_v4());
        let user_id: i64 = sqlx::query_scalar("INSERT INTO porte_users (email, name) VALUES ($1, $2) RETURNING id")
            .bind(format!("{subject}@example.test"))
            .bind(name)
            .fetch_one(&self.pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO porte_identities (user_id, provider, subject, refresh_token) VALUES ($1, $2, $3, $4)")
            .bind(user_id)
            .bind(ISSUER)
            .bind(&subject)
            .bind(format!("rt-{user_id}"))
            .execute(&self.pool)
            .await
            .unwrap();
        let (token, _) = self.kits.sessions.issue(user_id, "test").await.unwrap();
        Person { user_id, subject, token }
    }

    pub async fn space(&self, members: &[(&Person, &str)]) -> String {
        let id = format!("space-{}", uuid::Uuid::new_v4());
        sqlx::query("INSERT INTO spaces (id, name) VALUES ($1, $1)").bind(&id).execute(&self.pool).await.unwrap();
        for (person, role) in members {
            sqlx::query("INSERT INTO space_members (space_id, subject, email, role) VALUES ($1, $2, $2, $3)")
                .bind(&id)
                .bind(&person.subject)
                .bind(role)
                .execute(&self.pool)
                .await
                .unwrap();
        }
        id
    }

    pub async fn call(
        &self,
        method: &str,
        path: &str,
        who: Option<&Person>,
        body: Option<serde_json::Value>,
    ) -> Answer {
        send(&self.router, method, path, who.map(|p| p.token.as_str()), body).await
    }
}
