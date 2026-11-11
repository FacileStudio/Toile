mod common;

use std::time::Duration;

use common::board::{rect, ws};
use common::fixture;
use common::ysync::Peer;
use yrs::{Map, MapPrelim, Transact};

const QUIET: Duration = Duration::from_millis(400);

type Change = Box<dyn FnOnce(&mut yrs::TransactionMut, &yrs::MapRef)>;

async fn stored(pool: &sqlx::PgPool, id: &str) -> (i64, i64) {
    sqlx::query_as::<_, (i64, i64)>(
        "SELECT (SELECT count(*) FROM board_updates WHERE board_id = $1::uuid), \
         (SELECT version FROM board_state WHERE board_id = $1::uuid)",
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .unwrap()
}

#[tokio::test]
async fn two_clients_editing_offline_converge_and_a_fresh_client_reads_the_same() {
    let Some(pool) = common::pool("toile_rooms_converge").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let id = fixture::id(&fixture::board(&app, &roles.member, Some(&roles.space), "Live").await);
    let url = ws(&app.serve().await, &id);
    let (mut a, mut b) = (Peer::new(), Peer::new());
    a.connect(&url, Some(&roles.member.token)).await;
    b.connect(&url, Some(&roles.owner.token)).await;
    a.edit(|txn, shapes| rect(txn, shapes, "first", 1.0)).await;
    b.pump(QUIET).await;
    assert_eq!(b.shapes(), ["first"]);
    a.disconnect();
    b.disconnect();
    for i in 0..5 {
        a.edit(|txn, shapes| rect(txn, shapes, &format!("a{i}"), i as f64)).await;
        b.edit(|txn, shapes| rect(txn, shapes, &format!("b{i}"), i as f64)).await;
    }
    a.connect(&url, Some(&roles.member.token)).await;
    b.connect(&url, Some(&roles.owner.token)).await;
    a.pump(QUIET).await;
    assert_eq!(a.shapes(), b.shapes());
    assert_eq!(a.shapes().len(), 11);
    a.disconnect();
    b.disconnect();
    tokio::time::sleep(Duration::from_millis(300)).await;
    let mut fresh = Peer::new();
    fresh.connect(&url, Some(&roles.admin.token)).await;
    assert_eq!(fresh.shapes(), a.shapes());
}

fn bad_changes() -> Vec<Change> {
    vec![
        Box::new(|txn, shapes| _ = shapes.insert(txn, "x", MapPrelim::from([("type", "blob")]))),
        Box::new(|txn, shapes| {
            let fields =
                [("type", yrs::Any::from("rect")), ("x", 0.0.into()), ("y", 0.0.into()), ("w", f64::INFINITY.into())];
            shapes.insert(txn, "y", MapPrelim::from(fields));
        }),
        Box::new(|txn, shapes| {
            let fields =
                [("type", yrs::Any::from("ink")), ("x", 0.0.into()), ("y", 0.0.into()), ("points", vec![1.0].into())];
            shapes.insert(txn, "z", MapPrelim::from(fields));
        }),
    ]
}

#[tokio::test]
async fn an_invalid_update_is_reverted_and_its_sender_closed_with_1008() {
    let Some(pool) = common::pool("toile_rooms_invalid").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let id = fixture::id(&fixture::board(&app, &roles.member, None, "Safe").await);
    let url = ws(&app.serve().await, &id);
    let mut good = Peer::new();
    good.connect(&url, Some(&roles.member.token)).await;
    good.edit(|txn, shapes| rect(txn, shapes, "kept", 1.0)).await;
    good.pump(QUIET).await;
    for change in bad_changes() {
        let mut bad = Peer::new();
        bad.connect(&url, Some(&roles.member.token)).await;
        bad.edit(change).await;
        bad.pump(QUIET).await;
        assert_eq!(bad.closed, Some(1008));
    }
    good.pump(QUIET).await;
    assert_eq!(good.closed, None);
    assert_eq!(good.shapes(), ["kept"]);
    let mut fresh = Peer::new();
    fresh.connect(&url, Some(&roles.member.token)).await;
    assert_eq!(fresh.shapes(), ["kept"]);
    fresh.edit(|txn, shapes| rect(txn, shapes, "after", 2.0)).await;
    fresh.pump(QUIET).await;
    assert_eq!(fresh.closed, None);
}

#[tokio::test]
async fn an_update_creating_a_foreign_root_is_refused_with_1008() {
    let Some(pool) = common::pool("toile_rooms_root").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let id = fixture::id(&fixture::board(&app, &roles.member, None, "Roots").await);
    let url = ws(&app.serve().await, &id);
    let mut intruder = Peer::new();
    intruder.connect(&url, Some(&roles.member.token)).await;
    let root = intruder.doc.get_or_insert_map("intruder");
    intruder.edit(move |txn, _| _ = root.insert(txn, "k", 1)).await;
    intruder.pump(QUIET).await;
    assert_eq!(intruder.closed, Some(1008));
}

#[tokio::test]
async fn oversized_updates_and_frames_close_with_1009() {
    let Some(pool) = common::pool("toile_rooms_size").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let id = fixture::id(&fixture::board(&app, &roles.member, None, "Big").await);
    let url = ws(&app.serve().await, &id);
    let mut peer = Peer::new();
    peer.connect(&url, Some(&roles.member.token)).await;
    let big = "x".repeat((1 << 20) + 16);
    peer.edit(|txn, shapes| {
        shapes.insert(txn, "big", MapPrelim::from([("type", "card"), ("x", "0"), ("y", "0"), ("url", big.as_str())]));
    })
    .await;
    peer.pump(QUIET).await;
    assert_eq!(peer.closed, Some(1009));
    let mut frame = Peer::new();
    frame.connect(&url, Some(&roles.member.token)).await;
    frame.send_raw(vec![0u8; (2 << 20) + 1]).await;
    frame.pump(Duration::from_secs(2)).await;
    assert_eq!(frame.closed, Some(1009));
    let mut fresh = Peer::new();
    fresh.connect(&url, Some(&roles.member.token)).await;
    assert!(fresh.shapes().is_empty());
}

#[tokio::test]
async fn a_server_write_reaches_open_clients_and_an_invalid_one_is_refused() {
    let Some(pool) = common::pool("toile_rooms_write").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let id = fixture::id(&fixture::board(&app, &roles.member, None, "Agent").await);
    let url = ws(&app.serve().await, &id);
    let mut peer = Peer::new();
    peer.connect(&url, Some(&roles.member.token)).await;
    let board = uuid::Uuid::parse_str(&id).unwrap();
    let edit: api::modules::rooms::Edit = Box::new(|txn, shapes| {
        rect(txn, shapes, "from-server", 3.0);
        Ok(())
    });
    app.state.hub.write(&app.state, board, edit).await.unwrap();
    let bad: api::modules::rooms::Edit = Box::new(|txn, shapes| {
        shapes.insert(txn, "nope", MapPrelim::from([("type", "blob")]));
        Ok(())
    });
    let refused = app.state.hub.write(&app.state, board, bad).await.unwrap_err();
    assert_eq!(refused.code, tronc::Code::InvalidArgument);
    peer.pump(QUIET).await;
    assert_eq!(peer.shapes(), ["from-server"]);
    let state = app.state.hub.read(&app.state, board).await.unwrap();
    let doc = api::modules::board::from_state(&state, []).unwrap();
    assert_eq!(doc.get_or_insert_map("shapes").len(&doc.transact()), 1);
}

#[tokio::test]
async fn two_hundred_updates_compact_into_one_snapshot() {
    let Some(pool) = common::pool("toile_rooms_compact").await else { return };
    let app = common::app(pool.clone()).await;
    let roles = fixture::roles(&app).await;
    let id = fixture::id(&fixture::board(&app, &roles.member, None, "Busy").await);
    let url = ws(&app.serve().await, &id);
    let mut peer = Peer::new();
    peer.connect(&url, Some(&roles.member.token)).await;
    for i in 0..199 {
        peer.edit(|txn, shapes| rect(txn, shapes, &format!("s{i}"), i as f64)).await;
    }
    peer.pump(Duration::from_millis(600)).await;
    assert_eq!(stored(&pool, &id).await, (199, 1));
    peer.edit(|txn, shapes| rect(txn, shapes, "s199", 0.0)).await;
    peer.pump(Duration::from_millis(600)).await;
    assert_eq!(stored(&pool, &id).await, (0, 2));
}
