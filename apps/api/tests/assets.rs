mod common;

use api::auth::SpacesApply;
use common::directory::{base, snapshot, space};
use common::fixture;
use common::upload::{get, png, upload};
use porte::directory::Person;
use serde_json::json;

#[tokio::test]
async fn an_upload_lands_in_the_uploaders_app_folder_and_every_reader_sees_it() {
    let Some(pool) = common::pool("toile_assets").await else { return };
    let (nuage, apps) = common::fakes::start().await;
    let app = common::app_with(pool, apps).await;
    let roles = fixture::roles(&app).await;
    let board = fixture::id(&fixture::board(&app, &roles.owner, Some(&roles.space), "Moodboard").await);
    let path = format!("/api/boards/{board}/assets");
    let answer = upload(&app, &path, Some(&roles.member), &png(64)).await;
    assert_eq!(answer.status, 201, "{}", answer.text());
    let body = answer.json();
    let asset = body["asset"].as_str().unwrap().to_owned();
    assert_eq!((body["w"].clone(), body["h"].clone()), (json!(3), json!(2)));
    assert_eq!(body["url"], json!(format!("/api/boards/{board}/assets/{asset}")));
    let key = (roles.member.user_id.to_string(), format!("boards/{board}/{asset}.png"));
    assert_eq!(nuage.objects.lock().unwrap()[&key].0, "image/png");

    let read = get(&app, body["url"].as_str().unwrap(), Some(&roles.admin), None).await;
    assert_eq!((read.status.as_u16(), read.bytes.clone()), (200, png(64)));
    assert_eq!(read.headers["content-type"], "image/png");
    assert!(read.headers["cache-control"].to_str().unwrap().contains("immutable"));
    let etag = read.headers["etag"].to_str().unwrap().to_owned();
    assert_eq!(get(&app, body["url"].as_str().unwrap(), Some(&roles.owner), Some(&etag)).await.status, 304);
    assert_eq!(get(&app, body["url"].as_str().unwrap(), Some(&roles.outsider), None).await.status, 404);
    assert_eq!(upload(&app, &path, Some(&roles.outsider), &png(1)).await.status, 404);

    guests_read_but_never_upload(&app, &roles, &board, &asset).await;
}

async fn guests_read_but_never_upload(app: &common::App, roles: &fixture::Roles, board: &str, asset: &str) {
    let (_, token) = fixture::link(app, &roles.owner, board, "edit").await;
    let guest = get(app, &format!("/api/share/{token}/assets/{asset}"), None, None).await;
    assert_eq!((guest.status.as_u16(), guest.bytes), (200, png(64)));
    assert_eq!(upload(app, &format!("/api/share/{token}/assets"), None, &png(1)).await.status, 403);
    let other = fixture::id(&fixture::board(app, &roles.owner, Some(&roles.space), "Other").await);
    assert_eq!(get(app, &format!("/api/boards/{other}/assets/{asset}"), Some(&roles.owner), None).await.status, 404);
}

#[tokio::test]
async fn the_plan_caps_the_size_and_only_real_images_pass() {
    let Some(pool) = common::pool("toile_assets_plan").await else { return };
    let (_, apps) = common::fakes::start().await;
    let app = common::app_with(pool.clone(), apps).await;
    let ada = app.person("ada").await;
    let board = fixture::id(&fixture::board(&app, &ada, None, "Perso").await);
    let mut snap = snapshot(base(), vec![space("unused", &[])]);
    snap.entitlements_revision = 1;
    snap.people =
        vec![Person { subject: ada.subject.clone(), entitlements: [("toile.upload_mb".into(), json!(1))].into() }];
    let apply = SpacesApply::new(pool, app.state.hub.clone(), app.state.suite.plans.clone());
    porte::directory::Apply::apply(&apply, &snap).await.unwrap();
    let path = format!("/api/boards/{board}/assets");

    let refused = upload(&app, &path, Some(&ada), &png(1 << 20)).await;
    assert_eq!(refused.status, 402, "{}", refused.text());
    let error = &refused.json()["error"];
    assert_eq!(error["code"], "payment_required");
    assert_eq!((error["key"].clone(), error["limit"].clone()), (json!("toile.upload_mb"), json!(1)));
    assert_eq!(error["upgrade_url"], "https://sso.test/spaces");
    assert!(error["message"].as_str().unwrap().contains("1 MB"));

    assert_eq!(upload(&app, &path, Some(&ada), &png(1000)).await.status, 201);
    let svg =
        upload(&app, &path, Some(&ada), br#"<svg xmlns="http://www.w3.org/2000/svg"><script>x</script></svg>"#).await;
    assert_eq!(svg.status, 400);
    assert_eq!(upload(&app, &path, Some(&ada), b"plain text, not an image").await.status, 400);
}
