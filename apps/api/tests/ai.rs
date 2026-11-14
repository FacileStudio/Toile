mod common;

use api::auth::SpacesApply;
use common::bulle::{start, stickies};
use common::directory::{base, snapshot, space};
use common::fixture;
use porte::directory::Person;
use serde_json::json;

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
