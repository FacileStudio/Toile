use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use sqlx::PgPool;

pub struct Server {
    child: Child,
    pub port: u16,
}

pub fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port()
}

pub async fn start(database: &str, port: u16) -> Server {
    let child = Command::new(env!("CARGO_BIN_EXE_api"))
        .env("DATABASE_URL", database)
        .env("PORT", port.to_string())
        .env("LOG_LEVEL", "warn")
        .env_remove("OIDC_ISSUER")
        .env_remove("APP_ENV")
        .stdout(Stdio::null())
        .spawn()
        .unwrap();
    for _ in 0..600 {
        if reqwest::get(format!("http://127.0.0.1:{port}/health")).await.is_ok() {
            return Server { child, port };
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let mut child = child;
    child.kill().unwrap();
    child.wait().unwrap();
    panic!("the api did not start");
}

impl Server {
    pub fn kill(mut self) -> u16 {
        self.child.kill().unwrap();
        self.child.wait().unwrap();
        self.port
    }

    pub fn terminate(mut self) -> Duration {
        let started = Instant::now();
        let pid = self.child.id().to_string();
        assert!(Command::new("kill").args(["-TERM", &pid]).status().unwrap().success());
        self.child.wait().unwrap();
        started.elapsed()
    }
}

pub async fn database() -> Option<(String, PgPool)> {
    let admin = tronc::testdb::url()?;
    let name = format!("toile_kill_{}", uuid::Uuid::new_v4().simple());
    let pool = PgPool::connect(&admin).await.ok()?;
    sqlx::query(sqlx::AssertSqlSafe(format!("CREATE DATABASE {name}"))).execute(&pool).await.ok()?;
    let (base, query) = admin.split_once('?').map_or((admin.as_str(), ""), |(b, q)| (b, q));
    let url = format!("{}/{name}?{query}", base.rsplit_once('/').unwrap().0);
    Some((url.clone(), PgPool::connect(&url).await.ok()?))
}

pub async fn drop_database(url: &str) {
    let admin = PgPool::connect(&tronc::testdb::url().unwrap()).await.unwrap();
    let name = url.rsplit('/').next().unwrap().split('?').next().unwrap();
    sqlx::query(sqlx::AssertSqlSafe(format!("DROP DATABASE {name} WITH (FORCE)"))).execute(&admin).await.unwrap();
}
