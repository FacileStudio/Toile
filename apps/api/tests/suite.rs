mod common;

use std::collections::HashMap;
use std::sync::atomic::Ordering;

use api::auth::SpacesApply;
use api::modules::suite::Plans;
use common::directory::{base, snapshot, space};
use porte::directory::Person;
use porte::exchange::Target;
use serde_json::{Value, json};

fn values(ai: bool, mb: i64) -> HashMap<String, Value> {
    HashMap::from([("ai.access".to_owned(), json!(ai)), ("toile.upload_mb".to_owned(), json!(mb))])
}

#[tokio::test]
async fn entitlements_follow_the_snapshot_and_survive_a_restart() {
    let Some(pool) = common::pool("toile_suite_plans").await else { return };
    let app = common::app(pool.clone()).await;
    let ada = app.person("ada").await;
    let space_id = app.space(&[(&ada, "member")]).await;
    let open = app.call("GET", "/api/me/entitlements", Some(&ada), None).await.json();
    assert_eq!(open["entitlements"], json!({"ai.access": true, "toile.upload_mb": -1}), "fails open first");

    let mut snap = snapshot(base(), vec![space(&space_id, &[(&ada.subject, "member")])]);
    snap.entitlements_revision = 1;
    snap.spaces[0].entitlements = values(true, 50);
    snap.people = vec![Person { subject: ada.subject.clone(), entitlements: values(false, 20) }];
    let apply = SpacesApply::new(pool.clone(), app.state.hub.clone(), app.state.suite.plans.clone());
    porte::directory::Apply::apply(&apply, &snap).await.unwrap();

    let personal = app.call("GET", "/api/me/entitlements", Some(&ada), None).await.json();
    assert_eq!(personal["scope"], "personal");
    assert_eq!(personal["entitlements"], json!({"ai.access": false, "toile.upload_mb": 20}));
    assert_eq!(personal["upgrade_url"], "https://sso.test/spaces");
    let path = format!("/api/me/entitlements?space_id={space_id}");
    let shared = app.call("GET", &path, Some(&ada), None).await.json();
    assert_eq!(shared["entitlements"], json!({"ai.access": true, "toile.upload_mb": 50}));
    let outsider = app.person("eve").await;
    assert_eq!(app.call("GET", &path, Some(&outsider), None).await.status, 404);

    a_restart_restores_the_plans(pool, &ada.subject).await;
}

async fn a_restart_restores_the_plans(pool: sqlx::PgPool, subject: &str) {
    let restarted = Plans::new(pool.clone(), common::ISSUER);
    restarted.restore().await.unwrap();
    let person = porte::forfait::Scope::person(subject);
    assert_eq!(restarted.limit(&person, "toile.upload_mb"), 20);

    sqlx::query("UPDATE entitlement_state SET document = 'garbage'").execute(&pool).await.unwrap();
    let corrupt = Plans::new(pool, common::ISSUER);
    corrupt.restore().await.unwrap();
    assert_eq!(corrupt.limit(&person, "toile.upload_mb"), 10, "a corrupt row enforces the defaults");
    assert!(!corrupt.enabled(&person, "ai.access"));
}

#[tokio::test]
async fn signing_out_forgets_the_exchanged_tokens() {
    let Some(pool) = common::pool("toile_suite_logout").await else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let exchanger = app.state.suite.exchanger.clone().unwrap();
    let nuage = Target::new("nuage", ["nuage.write"]);
    let first = exchanger.token(ada.user_id, &nuage).await.unwrap();
    assert_eq!(common::registre::holder(&first), ada.user_id.to_string());
    assert_eq!(exchanger.token(ada.user_id, &nuage).await.unwrap(), first, "cached");
    let out = app.call("POST", "/api/auth/logout", Some(&ada), None).await;
    assert_eq!(out.status, 200, "{}", out.text());
    assert_ne!(exchanger.token(ada.user_id, &nuage).await.unwrap(), first);
    assert_eq!(app.registre.exchanges.load(Ordering::SeqCst), 2);
}
