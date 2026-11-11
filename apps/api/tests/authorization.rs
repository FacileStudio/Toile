mod common;

use common::fixture::{self, Roles};
use common::{App, Person};
use serde_json::{Value, json};

#[derive(Clone, Copy, PartialEq, PartialOrd)]
enum Need {
    Viewer,
    Editor,
}

struct Case {
    method: &'static str,
    path: &'static str,
    body: Option<fn() -> Value>,
    need: Need,
    ok: u16,
}

fn cases() -> Vec<Case> {
    vec![
        Case { method: "GET", path: "/api/boards/{id}", body: None, need: Need::Viewer, ok: 200 },
        Case { method: "GET", path: "/api/boards?space_id={space}", body: None, need: Need::Viewer, ok: 200 },
        Case {
            method: "POST",
            path: "/api/boards",
            body: Some(|| json!({"name": "Rétro", "space_id": "{space}"})),
            need: Need::Editor,
            ok: 201,
        },
    ]
}

async fn attempt(app: &App, roles: &Roles, case: &Case, who: Option<&Person>) -> u16 {
    let board = fixture::id(&fixture::board(app, &roles.owner, Some(&roles.space), "Atelier").await);
    let fill = |raw: &str| raw.replace("{id}", &board).replace("{space}", &roles.space);
    let body = case.body.map(|body| serde_json::from_str(&fill(&body().to_string())).unwrap());
    app.call(case.method, &fill(case.path), who, body).await.status.as_u16()
}

fn expected(case: &Case, have: Option<Need>) -> u16 {
    match have {
        None => 404,
        Some(have) if have < case.need => 403,
        Some(_) => case.ok,
    }
}

#[tokio::test]
async fn every_board_route_answers_each_role_as_its_access_allows() {
    let Some(pool) = common::pool("toile_authz").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let people = [
        (&roles.owner, Some(Need::Editor)),
        (&roles.admin, Some(Need::Editor)),
        (&roles.member, Some(Need::Editor)),
        (&roles.outsider, None),
    ];
    for case in cases() {
        assert_eq!(attempt(&app, &roles, &case, None).await, 401, "anonymous {} {}", case.method, case.path);
        for (person, have) in &people {
            let got = attempt(&app, &roles, &case, Some(person)).await;
            assert_eq!(got, expected(&case, *have), "{} {} as {:?}", case.method, case.path, person.subject);
        }
    }
}

#[tokio::test]
async fn a_personal_board_is_its_owners_alone() {
    let Some(pool) = common::pool("toile_personal").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let mine = fixture::id(&fixture::board(&app, &roles.member, None, "Brouillon").await);
    let path = format!("/api/boards/{mine}");
    let read = app.call("GET", &path, Some(&roles.member), None).await;
    assert_eq!(read.status, 200);
    assert_eq!(read.json()["access"], json!("manager"));
    for who in [&roles.owner, &roles.admin, &roles.outsider] {
        assert_eq!(app.call("GET", &path, Some(who), None).await.status, 404);
        let listed = app.call("GET", "/api/boards", Some(who), None).await.json();
        assert!(listed.as_array().unwrap().iter().all(|b| b["id"] != json!(mine)), "{listed}");
    }
    let other = format!("/api/boards?space_id={}", roles.other_space);
    assert_eq!(app.call("GET", &other, Some(&roles.member), None).await.status, 404);
}

#[tokio::test]
async fn malformed_ids_and_unknown_spaces_are_not_found() {
    let Some(pool) = common::pool("toile_missing").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    assert_eq!(app.call("GET", "/api/boards/nope", Some(&roles.owner), None).await.status, 404);
    let ghost = json!({"name": "x", "space_id": "no-such-space"});
    assert_eq!(app.call("POST", "/api/boards", Some(&roles.owner), Some(ghost)).await.status, 404);
}
