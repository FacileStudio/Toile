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
