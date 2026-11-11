mod common;

use std::time::Duration;

use common::board::ws;
use common::mcp::{Served, args, refused, served, shapes};
use common::ysync::Peer;
use serde_json::{Value, json};
use tronc::mcptest::{ClientOptions, call, connect, decode};

fn plan() -> Value {
    json!([
        {"type": "arrow", "from": "a", "to": "b"},
        {"type": "sticky", "ref": "a", "text": "Idea A", "frame": "f"},
        {"type": "sticky", "ref": "b", "text": "Idea B", "frame": "f", "color": "#fde68a"},
        {"type": "frame", "ref": "f", "text": "Ideas"}
    ])
}

#[tokio::test]
async fn agents_lay_out_a_frame_of_stickies_live_then_read_change_and_comment() {
    let Some(served) = served("toile_mcp_tools").await else { return };
    let id = served.board("Plan").await;
    let mut peer = Peer::new();
    peer.connect(&ws(&served.app.serve().await, &id), Some(&served.person.token)).await;
    let client = connect(&served.url, &served.token(), ClientOptions::default()).await;
    let listed: Value = decode(&call(&client, "toile_list_boards", args(json!({}))).await);
    assert_eq!(listed["boards"][0]["id"], json!(id));
    let added: Value =
        decode(&call(&client, "toile_add_shapes", args(json!({"board_id": id, "shapes": plan()}))).await);
    let created = added["created"].as_array().unwrap();
    let kinds: Vec<&str> = created.iter().map(|c| c["type"].as_str().unwrap()).collect();
    assert_eq!(kinds, ["frame", "sticky", "sticky", "arrow"]);
    peer.pump(Duration::from_millis(400)).await;
    assert_eq!(peer.shapes().len(), 4);
    let read: Value = decode(&call(&client, "toile_read_board", args(json!({"board_id": id}))).await);
    let text = read["shapes"].as_str().unwrap();
    assert!(text.starts_with(&format!("frame {} \"Ideas\"", created[0]["id"].as_str().unwrap())), "{text}");
    assert!(text.contains("  sticky") && text.contains("#fde68a: \"Idea B\""), "{text}");
    let a = created[1]["id"].as_str().unwrap();
    let change = json!({"board_id": id, "shapes": [{"id": a, "text": "Idea A2", "x": 5000, "y": 0}]});
    let changed: Value = decode(&call(&client, "toile_update_shapes", args(change)).await);
    assert!(changed["shapes"].as_str().unwrap().contains("at 5000,0 200x200: \"Idea A2\""), "{changed}");
    comment_on(&served, &client, &id, a).await;
}

async fn comment_on(served: &Served, client: &tronc::mcptest::Client, id: &str, shape: &str) {
    let note = json!({"board_id": id, "body": "Looks good", "shape_id": shape});
    let comment: Value = decode(&call(client, "toile_comment", args(note)).await);
    assert_eq!(comment["author"], json!("agent-owner"));
    let comments = served.app.call("GET", &format!("/api/boards/{id}/comments"), Some(&served.person), None).await;
    assert_eq!(comments.json()[0]["anchor"]["shape"], json!(shape));
}

#[tokio::test]
async fn bad_writes_and_strangers_are_refused_and_change_nothing() {
    let Some(served) = served("toile_mcp_refusals").await else { return };
    let id = served.board("Mine").await;
    let client = connect(&served.url, &served.token(), ClientOptions::default()).await;
    let ghost = json!({"board_id": id, "shapes": [{"type": "arrow", "from": "nope", "to": "nope"}]});
    assert!(refused(&client, "toile_add_shapes", ghost).await.contains("nope"));
    let huge = json!({"board_id": id, "shapes": [{"type": "sticky", "x": 1e9, "y": 0}]});
    assert!(refused(&client, "toile_add_shapes", huge).await.contains("within"));
    let missing = json!({"board_id": id, "shapes": [{"id": "ghost", "x": 1}]});
    assert!(refused(&client, "toile_update_shapes", missing).await.contains("ghost"));
    assert!(shapes(&served.app.state, &id).await.is_empty());
    let stranger = served.app.person("stranger").await;
    let theirs =
        connect(&served.url, &format!("{}|toile.read,toile.write", stranger.user_id), ClientOptions::default()).await;
    let add = json!({"board_id": id, "shapes": [{"type": "sticky"}]});
    assert!(refused(&theirs, "toile_add_shapes", add).await.contains("not found"));
    assert!(refused(&theirs, "toile_read_board", json!({"board_id": id})).await.contains("not found"));
}
