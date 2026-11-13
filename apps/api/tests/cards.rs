mod common;

use serde_json::{Value, json};

async fn resolve(app: &common::App, who: &common::Person, url: &str) -> (u16, Value) {
    let answer = app.call("POST", "/api/cards/resolve", Some(who), Some(json!({"url": url}))).await;
    let status = answer.status.as_u16();
    (status, if status == 200 { answer.json() } else { Value::Null })
}

#[tokio::test]
async fn suite_links_are_read_with_the_viewers_token_and_other_links_stay_bare() {
    let Some(pool) = common::pool("toile_cards").await else { return };
    let (fake, apps) = common::fakes::start().await;
    let (docs, opus, nuage) = (apps.docs.url.clone(), apps.opus.url.clone(), apps.nuage.url.clone());
    let app = common::app_with(pool, apps).await;
    let ada = app.person("ada").await;

    let doc = format!("{docs}/d/6f1e2d3c-4b5a-4968-8776-655443322110");
    let (status, card) = resolve(&app, &ada, &doc).await;
    assert_eq!(status, 200);
    assert_eq!(
        (card["kind"].clone(), card["title"].clone(), card["icon"].clone()),
        (json!("docs"), json!("Plan de rentrée"), json!("docs"))
    );
    resolve(&app, &ada, &doc).await;
    assert_eq!(fake.hits.lock().unwrap()["documents"], 1, "cached for ten minutes");

    let (_, hidden) = resolve(&app, &ada, &format!("{docs}/d/00000000-0000-4000-8000-000000000000")).await;
    assert_eq!((hidden["kind"].clone(), hidden["title"].clone()), (json!("docs"), json!("127.0.0.1")));
    let (_, task) = resolve(&app, &ada, &format!("{opus}/dashboard/workspace/3/project/7/task/42")).await;
    assert_eq!((task["title"].clone(), task["status"].clone()), (json!("Ship it"), json!("in_progress")));
    let (_, file) = resolve(&app, &ada, &format!("{nuage}/files/9")).await;
    assert_eq!((file["kind"].clone(), file.get("subtitle")), (json!("nuage"), None));
    let (_, sneaky) = resolve(&app, &ada, &format!("{docs}/d/..%2f..%2fadmin")).await;
    assert_eq!(sneaky["title"], "127.0.0.1");

    for url in ["https://127.0.0.1/secret", "https://10.0.0.1/", "http://example.test/"] {
        let (status, card) = resolve(&app, &ada, url).await;
        assert_eq!((status, card["kind"].clone()), (200, json!("link")), "{url}");
    }
    assert_eq!(resolve(&app, &ada, "javascript:alert(1)").await.0, 400);
    assert_eq!(resolve(&app, &ada, "not a url").await.0, 400);
    assert_eq!(app.call("POST", "/api/cards/resolve", None, Some(json!({"url": doc}))).await.status, 401);
}

#[tokio::test]
async fn youtube_links_are_resolved_with_thumbnails_and_kind() {
    let Some(pool) = common::pool("toile_cards_yt").await else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;

    let (status, card) = resolve(&app, &ada, "https://www.youtube.com/watch?v=dQw4w9WgXcQ").await;
    assert_eq!(status, 200);
    assert_eq!(card["kind"], "youtube");
    assert_eq!(card["icon"], "youtube");
    assert_eq!(card["site"], "YouTube");
    assert_eq!(card["image"], "https://img.youtube.com/vi/dQw4w9WgXcQ/hqdefault.jpg");
    assert_eq!(card["favicon"], "https://www.google.com/s2/favicons?domain=www.youtube.com&sz=64");

    let (_, short) = resolve(&app, &ada, "https://youtu.be/dQw4w9WgXcQ").await;
    assert_eq!(short["kind"], "youtube");
    assert_eq!(short["image"], "https://img.youtube.com/vi/dQw4w9WgXcQ/hqdefault.jpg");

    let (_, shorts) = resolve(&app, &ada, "https://www.youtube.com/shorts/dQw4w9WgXcQ").await;
    assert_eq!(shorts["kind"], "youtube");
    assert_eq!(shorts["image"], "https://img.youtube.com/vi/dQw4w9WgXcQ/hqdefault.jpg");

    let (_, embed) = resolve(&app, &ada, "https://www.youtube.com/embed/dQw4w9WgXcQ").await;
    assert_eq!(embed["kind"], "youtube");
    assert_eq!(embed["image"], "https://img.youtube.com/vi/dQw4w9WgXcQ/hqdefault.jpg");
}

#[test]
fn og_metadata_serialization_skips_none_fields() {
    let url = url::Url::parse("https://example.com/page").unwrap();
    let bare = api::modules::cards::Card::link(&url);
    let bare_json = serde_json::to_value(&bare).unwrap();
    assert_eq!(bare_json["kind"], "link");
    assert_eq!(bare_json["title"], "example.com");
    assert!(bare_json.get("image").is_none());
    assert!(bare_json.get("favicon").is_none());
    assert!(bare_json.get("site").is_none());

    let mut full = bare;
    full.image = Some("https://example.com/og.png".to_string());
    full.favicon = Some("https://example.com/fav.ico".to_string());
    full.site = Some("Example Site".to_string());
    let full_json = serde_json::to_value(&full).unwrap();
    assert_eq!(full_json["image"], "https://example.com/og.png");
    assert_eq!(full_json["favicon"], "https://example.com/fav.ico");
    assert_eq!(full_json["site"], "Example Site");
}

const PASTED: &str = r#"{"type":"card","x":-1652.25,"y":-877.009765625,"w":320,"h":200,"rot":0,"z":"Y","style":{},
"kind":"youtube","url":"https://youtu.be/PAy5EBTnV6Q?list=RDERCkf0j8pPM","title":"youtu.be",
"image":"https://img.youtube.com/vi/PAy5EBTnV6Q/hqdefault.jpg",
"favicon":"https://www.google.com/s2/favicons?domain=youtu.be&sz=64","site":"YouTube","description":"A video"}"#;

#[tokio::test]
async fn a_pasted_youtube_card_with_its_preview_is_kept_by_the_room() {
    let Some(pool) = common::pool("toile_cards_pasted").await else { return };
    let app = common::app(pool).await;
    let roles = common::fixture::roles(&app).await;
    let id = common::fixture::id(&common::fixture::board(&app, &roles.member, None, "Cards").await);
    let url = common::board::ws(&app.serve().await, &id);
    let mut peer = common::ysync::Peer::new();
    peer.connect(&url, Some(&roles.member.token)).await;
    let yrs::Any::Map(fields) = yrs::Any::from_json(PASTED).unwrap() else { panic!("not an object") };
    peer.edit(|txn, shapes| {
        let prelim: yrs::MapPrelim = fields.iter().map(|(k, v)| (k.as_str(), v.clone())).collect();
        yrs::Map::insert(shapes, txn, "ceS6wM59r4pIHwfY", prelim);
    })
    .await;
    peer.pump(std::time::Duration::from_millis(400)).await;
    assert_eq!(peer.closed, None);
    let mut fresh = common::ysync::Peer::new();
    fresh.connect(&url, Some(&roles.member.token)).await;
    assert_eq!(fresh.shapes(), ["ceS6wM59r4pIHwfY"]);
}
