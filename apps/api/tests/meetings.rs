mod common;

use serde_json::{Value, json};

async fn meet(app: &common::App, who: &common::Person, body: Value) -> (u16, Value) {
    let answer = app.call("POST", "/api/boards/meeting", Some(who), Some(body)).await;
    (answer.status.as_u16(), answer.json())
}

#[tokio::test]
async fn an_echo_room_gets_one_board_and_one_edit_link_per_person() {
    let Some(pool) = common::pool("toile_meetings").await else { return };
    let app = common::app(pool).await;
    let (host, other) = (app.person("host").await, app.person("other").await);
    let (status, first) = meet(&app, &host, json!({"room": "echo-42", "name": "Weekly"})).await;
    assert_eq!((status, first["created"].clone(), first["board"]["name"].clone()), (201, json!(true), json!("Weekly")));
    let (status, again) = meet(&app, &host, json!({"room": " echo-42 "})).await;
    assert_eq!(
        (status, again["board"]["id"].clone(), again["edit_link"].clone()),
        (200, first["board"]["id"].clone(), first["edit_link"].clone())
    );
    let token = first["edit_link"].as_str().unwrap().trim_start_matches("/s/").to_owned();
    let shared = app.call("GET", &format!("/api/share/{token}"), None, None).await.json();
    assert_eq!((shared["role"].clone(), shared["board"]["id"].clone()), (json!("edit"), first["board"]["id"].clone()));
    let (_, theirs) = meet(&app, &other, json!({"room": "echo-42"})).await;
    assert_ne!(theirs["board"]["id"], first["board"]["id"]);
}

#[tokio::test]
async fn a_trashed_meeting_board_is_replaced_and_bad_rooms_are_refused() {
    let Some(pool) = common::pool("toile_meetings_trash").await else { return };
    let app = common::app(pool).await;
    let host = app.person("host").await;
    let (_, first) = meet(&app, &host, json!({"room": "r1"})).await;
    let id = first["board"]["id"].as_str().unwrap();
    assert_eq!(app.call("DELETE", &format!("/api/boards/{id}"), Some(&host), None).await.status, 204);
    let (status, next) = meet(&app, &host, json!({"room": "r1"})).await;
    assert_eq!((status, next["board"]["name"].clone()), (201, json!("Réunion")));
    assert_ne!(next["board"]["id"], first["board"]["id"]);
    assert_eq!(meet(&app, &host, json!({"room": "  "})).await.0, 400);
    assert_eq!(meet(&app, &host, json!({"room": "a\u{7}b"})).await.0, 400);
    assert_eq!(app.call("POST", "/api/boards/meeting", None, Some(json!({"room": "x"}))).await.status.as_u16(), 401);
}
