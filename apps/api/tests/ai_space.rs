mod common;

use api::auth::SpacesApply;
use common::bulle::{start, stickies};
use common::directory::{base, snapshot, space};
use common::fixture::{self, Roles};
use porte::directory::{Member, Person};
use serde_json::json;

async fn plans(app: &common::App, roles: &Roles, revision: i64, in_space: bool, own: bool) {
    let mut shared = space(&roles.space, &[]);
    shared.members = [(&roles.owner, "owner"), (&roles.member, "member")]
        .map(|(p, role)| Member { subject: p.subject.clone(), role: role.into(), ..Member::default() })
        .into();
    shared.entitlements = [("ai.access".into(), json!(in_space))].into();
    let mut snap = snapshot(base(), vec![shared]);
    snap.entitlements_revision = revision;
    snap.people =
        vec![Person { subject: roles.member.subject.clone(), entitlements: [("ai.access".into(), json!(own))].into() }];
    let apply = SpacesApply::new(app.pool.clone(), app.state.hub.clone(), app.state.suite.plans.clone());
    porte::directory::Apply::apply(&apply, &snap).await.unwrap();
}

async fn boards(app: &common::App, roles: &Roles) -> (String, String) {
    let personal = fixture::id(&fixture::board(app, &roles.member, None, "Perso").await);
    let shared = fixture::id(&fixture::board(app, &roles.owner, Some(&roles.space), "Atelier").await);
    stickies(app, &personal, &["Une idée"]).await;
    stickies(app, &shared, &["Une idée"]).await;
    (personal, shared)
}

async fn summarize(app: &common::App, roles: &Roles, board: &str) -> u16 {
    app.call("POST", &format!("/api/boards/{board}/ai/summarize"), Some(&roles.member), None).await.status.as_u16()
}

#[tokio::test]
async fn bulle_hears_the_space_only_for_a_space_board() {
    let Some(pool) = common::pool("toile_ai_space_id").await else { return };
    let (bulle, apps) = start().await;
    let app = common::app_with(pool, apps).await;
    let roles = fixture::roles(&app).await;
    let (personal, shared) = boards(&app, &roles).await;
    assert_eq!(summarize(&app, &roles, &personal).await, 200);
    assert_eq!(summarize(&app, &roles, &shared).await, 200);
    let calls = bulle.calls.lock().unwrap().clone();
    assert_eq!(calls[0].1.get("space_id"), None, "{}", calls[0].1);
    assert_eq!(calls[1].1["space_id"], json!(roles.space));
}

#[tokio::test]
async fn ai_access_comes_from_the_own_plan_or_the_space_plan() {
    let Some(pool) = common::pool("toile_ai_space_plan").await else { return };
    let (_, apps) = start().await;
    let app = common::app_with(pool, apps).await;
    let roles = fixture::roles(&app).await;
    let (personal, shared) = boards(&app, &roles).await;
    plans(&app, &roles, 1, true, false).await;
    assert_eq!(summarize(&app, &roles, &shared).await, 200, "the space plan covers its boards");
    assert_eq!(summarize(&app, &roles, &personal).await, 402, "never a personal board");
    plans(&app, &roles, 2, false, true).await;
    assert_eq!(summarize(&app, &roles, &shared).await, 200, "the own plan covers a space board");
    assert_eq!(summarize(&app, &roles, &personal).await, 200);
    plans(&app, &roles, 3, false, false).await;
    assert_eq!(summarize(&app, &roles, &shared).await, 402);
}
