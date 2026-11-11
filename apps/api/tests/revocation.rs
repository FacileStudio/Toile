mod common;

use std::time::Duration;

use api::auth::{AppData, SpacesApply};
use common::board::{rect, ws};
use common::fixture;
use common::ysync::Peer;
use porte::directory::PersonDeleted;

const QUIET: Duration = Duration::from_millis(600);

#[tokio::test]
async fn a_socket_closes_with_1008_once_its_session_is_logged_out() {
    let Some(pool) = common::pool("toile_revoke_logout").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let id = fixture::id(&fixture::board(&app, &roles.member, Some(&roles.space), "Open").await);
    let url = ws(&app.serve().await, &id);
    let (mut gone, mut stays) = (Peer::new(), Peer::new());
    gone.connect(&url, Some(&roles.member.token)).await;
    stays.connect(&url, Some(&roles.owner.token)).await;
    sqlx::query("DELETE FROM porte_sessions WHERE user_id = $1")
        .bind(roles.member.user_id)
        .execute(&app.pool)
        .await
        .unwrap();
    gone.pump(QUIET).await;
    stays.pump(Duration::from_millis(200)).await;
    assert_eq!((gone.closed, stays.closed), (Some(1008), None));
}

#[tokio::test]
async fn a_space_removed_by_the_directory_closes_its_sockets_and_refuses_reconnects() {
    let Some(pool) = common::pool("toile_revoke_space").await else { return };
    let app = common::app(pool.clone()).await;
    let roles = fixture::roles(&app).await;
    let id = fixture::id(&fixture::board(&app, &roles.member, Some(&roles.space), "Team").await);
    let url = ws(&app.serve().await, &id);
    let mut peer = Peer::new();
    peer.connect(&url, Some(&roles.member.token)).await;
    let apply = SpacesApply::new(pool, app.state.hub.clone());
    let other = porte::directory::Space { id: "elsewhere".into(), ..Default::default() };
    let snapshot = common::directory::snapshot(common::directory::base(), vec![other]);
    assert!(apply.apply_snapshot(&snapshot).await.unwrap());
    peer.pump(QUIET).await;
    assert_eq!(peer.closed, Some(1008));
    let status = app.call("GET", &format!("/api/boards/{id}/ws"), Some(&roles.member), None).await.status;
    assert_eq!(status, 404);
}

#[tokio::test]
async fn a_member_dropped_from_a_space_is_evicted_while_others_stay() {
    let Some(pool) = common::pool("toile_revoke_member").await else { return };
    let app = common::app(pool.clone()).await;
    let roles = fixture::roles(&app).await;
    let id = fixture::id(&fixture::board(&app, &roles.owner, Some(&roles.space), "Team").await);
    let url = ws(&app.serve().await, &id);
    let (mut dropped, mut kept) = (Peer::new(), Peer::new());
    dropped.connect(&url, Some(&roles.member.token)).await;
    kept.connect(&url, Some(&roles.owner.token)).await;
    let space = common::directory::space(&roles.space, &[(&roles.owner.subject, "owner")]);
    let snapshot = common::directory::snapshot(common::directory::base(), vec![space]);
    assert!(SpacesApply::new(pool, app.state.hub.clone()).apply_snapshot(&snapshot).await.unwrap());
    dropped.pump(QUIET).await;
    kept.pump(Duration::from_millis(200)).await;
    assert_eq!((dropped.closed, kept.closed), (Some(1008), None));
}

#[tokio::test]
async fn erasing_a_person_closes_their_sockets_and_keeps_their_last_shared_edit() {
    let Some(pool) = common::pool("toile_revoke_erase").await else { return };
    let app = common::app(pool.clone()).await;
    let roles = fixture::roles(&app).await;
    let personal = fixture::id(&fixture::board(&app, &roles.member, None, "Mine").await);
    let shared = fixture::id(&fixture::board(&app, &roles.member, Some(&roles.space), "Ours").await);
    let base = app.serve().await;
    let (mut own, mut erased, mut other) = (Peer::new(), Peer::new(), Peer::new());
    own.connect(&ws(&base, &personal), Some(&roles.member.token)).await;
    erased.connect(&ws(&base, &shared), Some(&roles.member.token)).await;
    other.connect(&ws(&base, &shared), Some(&roles.owner.token)).await;
    erased.edit(|txn, shapes| rect(txn, shapes, "last", 1.0)).await;
    erased.pump(Duration::from_millis(200)).await;
    AppData::new(pool.clone(), app.state.hub.clone())
        .on_person_deleted(common::ISSUER, &roles.member.subject)
        .await
        .unwrap();
    own.pump(QUIET).await;
    erased.pump(QUIET).await;
    assert_eq!((own.closed, erased.closed), (Some(1008), Some(1008)));
    other.disconnect();
    tokio::time::sleep(Duration::from_millis(300)).await;
    let mut fresh = Peer::new();
    fresh.connect(&ws(&base, &shared), Some(&roles.owner.token)).await;
    assert_eq!(fresh.shapes(), ["last"]);
    let left: i64 = sqlx::query_scalar("SELECT count(*) FROM boards WHERE id = $1::uuid")
        .bind(&personal)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(left, 0);
}

#[tokio::test]
async fn a_cookie_upgrade_from_another_origin_is_refused() {
    let Some(pool) = common::pool("toile_revoke_origin").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let id = fixture::id(&fixture::board(&app, &roles.member, None, "x").await);
    let url = ws(&app.serve().await, &id);
    use tokio_tungstenite::tungstenite::client::IntoClientRequest;
    let mut request = url.as_str().into_client_request().unwrap();
    request.headers_mut().insert("cookie", format!("session={}", roles.member.token).parse().unwrap());
    request.headers_mut().insert("origin", "https://evil.test".parse().unwrap());
    match tokio_tungstenite::connect_async(request).await {
        Err(tokio_tungstenite::tungstenite::Error::Http(response)) => assert_eq!(response.status(), 403),
        other => panic!("cross-origin upgrade not refused: {:?}", other.map(|(_, r)| r.status())),
    }
}
