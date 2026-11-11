mod common;

use api::auth::{AppData, SpaceStore, SpacesApply};
use common::directory::{base, snapshot, space};
use common::fixture;
use porte::directory::PersonDeleted;
use serde_json::json;

#[tokio::test]
async fn the_space_store_passes_porte_conformance() {
    let Some(pool) = common::pool("toile_conformance").await else { return };
    let result = porte::spaces::conformance::run(|| {
        let pool = pool.clone();
        async move {
            sqlx::query("TRUNCATE spaces CASCADE").execute(&pool).await.unwrap();
            SpaceStore::new(pool)
        }
    })
    .await;
    assert!(result.is_ok(), "{:#?}", result.unwrap_err());
}

#[tokio::test]
async fn a_snapshot_replaces_spaces_and_members_once_per_version() {
    let Some(pool) = common::pool("toile_directory").await else { return };
    let apply = SpacesApply::new(pool.clone());
    let version = base();
    assert!(
        apply.apply_snapshot(&snapshot(version, vec![space("a", &[("s1", "owner")]), space("b", &[])])).await.unwrap()
    );
    assert!(!apply.apply_snapshot(&snapshot(version, vec![space("x", &[])])).await.unwrap());
    assert!(apply.apply_snapshot(&snapshot(version + 1, vec![space("a", &[("s2", "member")])])).await.unwrap());
    let spaces: Vec<String> = sqlx::query_scalar("SELECT id FROM spaces WHERE deleted_at IS NULL ORDER BY id")
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(spaces, ["a"]);
    let members: Vec<String> = sqlx::query_scalar("SELECT subject FROM space_members").fetch_all(&pool).await.unwrap();
    assert_eq!(members, ["s2"]);
}

#[tokio::test]
async fn a_space_missing_from_a_snapshot_takes_its_boards_with_it() {
    let Some(pool) = common::pool("toile_dir_retire").await else { return };
    let app = common::app(pool.clone()).await;
    let roles = fixture::roles(&app).await;
    let board = fixture::id(&fixture::board(&app, &roles.owner, Some(&roles.space), "Plan").await);
    let apply = SpacesApply::new(pool.clone());
    let version = base();
    assert!(apply.apply_snapshot(&snapshot(version, vec![space("other", &[])])).await.unwrap());
    for who in [&roles.owner, &roles.member] {
        assert_eq!(app.call("GET", &format!("/api/boards/{board}"), Some(who), None).await.status, 404);
    }
    assert_eq!(app.call("GET", "/api/spaces", Some(&roles.owner), None).await.json(), json!({"spaces": []}));
    let back = space(&roles.space, &[(&roles.owner.subject, "owner")]);
    assert!(apply.apply_snapshot(&snapshot(version + 1, vec![back])).await.unwrap());
    assert_eq!(app.call("GET", &format!("/api/boards/{board}"), Some(&roles.owner), None).await.status, 404);
}

#[tokio::test]
async fn an_empty_snapshot_is_refused_while_spaces_exist() {
    let Some(pool) = common::pool("toile_dir_empty").await else { return };
    let app = common::app(pool.clone()).await;
    let roles = fixture::roles(&app).await;
    assert!(SpacesApply::new(pool.clone()).apply_snapshot(&snapshot(base(), vec![])).await.is_err());
    let listed = app.call("GET", "/api/spaces", Some(&roles.member), None).await.json();
    assert_eq!(listed["spaces"][0]["id"], json!(roles.space));
}

#[tokio::test]
async fn erasing_a_person_removes_their_personal_boards_memberships_and_account() {
    let Some(pool) = common::pool("toile_erase").await else { return };
    let app = common::app(pool.clone()).await;
    let roles = fixture::roles(&app).await;
    let personal = fixture::id(&fixture::board(&app, &roles.member, None, "Mien").await);
    let shared = fixture::id(&fixture::board(&app, &roles.member, Some(&roles.space), "Nôtre").await);
    AppData::new(pool.clone()).on_person_deleted(common::ISSUER, &roles.member.subject).await.unwrap();
    let ids = [uuid::Uuid::parse_str(&personal).unwrap(), uuid::Uuid::parse_str(&shared).unwrap()];
    let left: Vec<uuid::Uuid> =
        sqlx::query_scalar("SELECT id FROM boards WHERE id = ANY($1)").bind(&ids[..]).fetch_all(&pool).await.unwrap();
    assert_eq!(left, [ids[1]]);
    assert_eq!(app.call("GET", "/api/spaces", Some(&roles.member), None).await.status, 401);
    assert_eq!(app.call("GET", &format!("/api/boards/{shared}"), Some(&roles.owner), None).await.status, 200);
}
