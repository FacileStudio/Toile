mod common;

use common::mcp::{Served, args, refused, served};
use serde_json::{Value, json};
use tronc::mcptest::{Client, ClientOptions, call, connect, decode};

async fn decoded(client: &Client, tool: &str, input: Value) -> Value {
    decode(&call(client, tool, args(input)).await)
}

#[tokio::test]
async fn every_tool_is_annotated_honestly() {
    let Some(served) = served("toile_mcp_notes").await else { return };
    let client = connect(&served.url, &served.token(), ClientOptions::default()).await;
    let tools = client.list_all_tools().await.unwrap();
    let hint = |name: &str| {
        let tool = tools.iter().find(|t| t.name == name).unwrap_or_else(|| panic!("{name} is missing"));
        let a = tool.annotations.clone().unwrap();
        (a.read_only_hint.unwrap(), a.destructive_hint.unwrap(), a.idempotent_hint.unwrap(), a.open_world_hint.unwrap())
    };
    for read in ["toile_list_boards", "toile_read_board", "toile_list_comments"] {
        assert_eq!(hint(read), (true, false, true, false), "{read}");
    }
    for overwrite in ["toile_update_shapes", "toile_update_board", "toile_trash_board"] {
        assert_eq!(hint(overwrite), (false, true, true, false), "{overwrite}");
    }
    assert_eq!(hint("toile_delete_shapes"), (false, true, false, false));
    assert_eq!(hint("toile_create_board"), (false, false, false, false));
    assert_eq!(hint("toile_restore_board"), (false, false, true, false));
    assert_eq!(hint("toile_resolve_comment"), (false, true, true, false));
    assert_eq!(tools.len(), 12);
}

#[tokio::test]
async fn agents_create_rename_move_trash_and_restore_a_board() {
    let Some(served) = served("toile_mcp_manage").await else { return };
    let client = connect(&served.url, &served.token(), ClientOptions::default()).await;
    let space = served.app.space(&[(&served.person, "owner")]).await;
    let made = decoded(&client, "toile_create_board", json!({"title": "Atelier", "space_id": space})).await;
    let id = made["id"].as_str().unwrap().to_owned();
    assert_eq!((made["name"].clone(), made["space_id"].clone()), (json!("Atelier"), json!(space)));
    let moved = json!({"board_id": id, "name": "Atelier perso", "move_to": "personal"});
    let moved = decoded(&client, "toile_update_board", moved).await;
    assert_eq!((moved["name"].clone(), moved["space_id"].clone()), (json!("Atelier perso"), Value::Null));
    let back = decoded(&client, "toile_update_board", json!({"board_id": id, "move_to": space})).await;
    assert_eq!(back["space_id"], json!(space));
    for _ in 0..2 {
        let trashed = decoded(&client, "toile_trash_board", json!({"board_id": id})).await;
        assert_eq!(trashed["trashed"], json!(true));
    }
    assert!(refused(&client, "toile_read_board", json!({"board_id": id})).await.contains("not found"));
    for _ in 0..2 {
        let restored = decoded(&client, "toile_restore_board", json!({"board_id": id})).await;
        assert_eq!((restored["trashed"].clone(), restored["name"].clone()), (json!(false), json!("Atelier perso")));
    }
    paging(&served, &client).await;
}

async fn paging(served: &Served, client: &Client) {
    for name in ["A", "B"] {
        served.board(name).await;
    }
    let first = decoded(client, "toile_list_boards", json!({"limit": 2})).await;
    assert_eq!(first["boards"].as_array().unwrap().len(), 2);
    let cursor = first["next_cursor"].as_str().unwrap().to_owned();
    let rest = decoded(client, "toile_list_boards", json!({"limit": 2, "cursor": cursor})).await;
    assert_eq!((rest["boards"].as_array().unwrap().len(), rest["next_cursor"].clone()), (1, Value::Null));
}

#[tokio::test]
async fn members_may_not_move_or_trash_and_bad_input_names_the_field() {
    let Some(served) = served("toile_mcp_manage_refusals").await else { return };
    let member = served.app.person("member").await;
    let space = served.app.space(&[(&served.person, "owner"), (&member, "member")]).await;
    let theirs =
        connect(&served.url, &format!("{}|toile.read,toile.write", member.user_id), ClientOptions::default()).await;
    let made = decoded(&theirs, "toile_create_board", json!({"name": "Shared", "space_id": space})).await;
    let id = made["id"].as_str().unwrap();
    let owner = connect(&served.url, &served.token(), ClientOptions::default()).await;
    assert!(refused(&theirs, "toile_trash_board", json!({"board_id": id})).await.contains("may not"));
    let away = json!({"board_id": id, "move_to": "personal"});
    assert!(refused(&theirs, "toile_update_board", away).await.contains("may not"));
    assert!(refused(&theirs, "toile_update_board", json!({"board_id": id})).await.contains("move_to"));
    let nowhere = json!({"board_id": id, "move_to": "space-ghost"});
    assert!(refused(&owner, "toile_update_board", nowhere).await.contains("move_to takes a space id"));
    let renamed = decoded(&theirs, "toile_update_board", json!({"board_id": id, "name": "Renamed"})).await;
    assert_eq!((renamed["name"].clone(), renamed["space_id"].clone()), (json!("Renamed"), json!(space)));
    assert!(refused(&owner, "toile_restore_board", json!({"board_id": "x"})).await.contains("toile_list_boards"));
}

#[tokio::test]
async fn agents_list_open_threads_and_resolve_them_live() {
    let Some(served) = served("toile_mcp_threads").await else { return };
    let id = served.board("Retro").await;
    let client = connect(&served.url, &served.token(), ClientOptions::default()).await;
    let mut threads = Vec::new();
    for (n, body) in ["Premier", "Second", "Troisième"].iter().enumerate() {
        let note = json!({"board_id": id, "body": body, "x": n as f64 * 100.0, "y": 0});
        threads.push(decoded(&client, "toile_comment", note).await["id"].as_str().unwrap().to_owned());
    }
    decoded(&client, "toile_comment", json!({"board_id": id, "body": "Réponse", "reply_to": threads[0]})).await;
    let resolve = json!({"board_id": id, "comment_id": threads[1]});
    for _ in 0..2 {
        assert_eq!(decoded(&client, "toile_resolve_comment", resolve.clone()).await["resolved"], json!(true));
    }
    let open = decoded(&client, "toile_list_comments", json!({"board_id": id, "limit": 1})).await;
    assert_eq!(open["threads"][0]["body"], json!("Premier"));
    assert_eq!(open["threads"][0]["replies"][0]["body"], json!("Réponse"));
    let cursor = open["next_cursor"].as_str().unwrap();
    let next = decoded(&client, "toile_list_comments", json!({"board_id": id, "cursor": cursor})).await;
    assert_eq!((next["threads"][0]["body"].clone(), next["next_cursor"].clone()), (json!("Troisième"), Value::Null));
    let done = decoded(&client, "toile_list_comments", json!({"board_id": id, "status": "resolved"})).await;
    assert_eq!(done["threads"].as_array().unwrap().len(), 1);
    let stranger = served.app.person("stranger").await;
    let theirs = connect(&served.url, &format!("{}|toile.read", stranger.user_id), ClientOptions::default()).await;
    assert!(refused(&theirs, "toile_list_comments", json!({"board_id": id})).await.contains("not found"));
    let bad = json!({"board_id": id, "comment_id": "nope"});
    assert!(refused(&client, "toile_resolve_comment", bad).await.contains("toile_list_comments"));
}

#[tokio::test]
async fn long_threads_page_within_the_result_budget() {
    let Some(served) = served("toile_mcp_budget").await else { return };
    let id = served.board("Long").await;
    let client = connect(&served.url, &served.token(), ClientOptions::default()).await;
    let body = "é".repeat(4_500);
    let mut threads = Vec::new();
    for n in 0..10 {
        let note = json!({"board_id": id, "body": body, "x": n as f64 * 100.0, "y": 0});
        threads.push(decoded(&client, "toile_comment", note).await["id"].clone());
    }
    for _ in 0..9 {
        decoded(&client, "toile_comment", json!({"board_id": id, "body": body, "reply_to": threads[0]})).await;
    }
    let raw = call(&client, "toile_list_comments", args(json!({"board_id": id}))).await;
    assert!(tronc::mcptest::text(&raw).len() <= tronc::mcp::RESULT_BUDGET + 1_000);
    let open: Value = decode(&raw);
    let replies = open["threads"][0]["replies"].as_array().unwrap().len();
    assert!(replies < 9 && open["threads"][0]["replies_truncated"] == json!(true));
    let shown = open["threads"].as_array().unwrap().len();
    assert!(shown < 10);
    let cursor = open["next_cursor"].as_str().expect("a cut page names where to resume");
    let rest = decoded(&client, "toile_list_comments", json!({"board_id": id, "cursor": cursor, "limit": 200})).await;
    assert_eq!(rest["threads"][0]["replies_truncated"], Value::Null);
    assert_eq!(rest["threads"][0]["body"], json!(body));
}
