use serde_json::Value;
use url::Url;

use super::Card;
use crate::modules::suite::App;
use crate::state::AppState;

/// A link into a suite app: which app, and the API path that describes the item.
struct Item<'a> {
    app: &'a App,
    api: String,
}

fn segments(url: &Url) -> Vec<&str> {
    url.path_segments().map(|parts| parts.filter(|part| !part.is_empty()).collect()).unwrap_or_default()
}

fn item<'a>(state: &'a AppState, url: &Url) -> Option<Result<Item<'a>, &'a App>> {
    let apps = &state.suite.apps;
    let app =
        [&apps.docs, &apps.feuilles, &apps.opus, &apps.agenda, &apps.nuage].into_iter().find(|app| app.owns(url))?;
    let path = segments(url);
    let event = url.query_pairs().find(|(key, _)| key == "event").map(|(_, value)| value.into_owned());
    let api = match (app.key, path.as_slice()) {
        ("docs", ["d", id, ..]) => format!("/api/documents/{id}"),
        ("feuilles", ["w", id, ..]) => format!("/api/workbooks/{id}"),
        ("opus", ["dashboard", "workspace", _, "project", _, "task", id, ..]) => format!("/api/task/{id}"),
        ("agenda", ["events", id, ..]) => format!("/api/events/{id}"),
        ("agenda", _) if event.is_some() => format!("/api/events/{}", event.unwrap_or_default()),
        _ => return Some(Err(app)),
    };
    let safe = api.split('/').all(|part| part.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'));
    Some(if safe { Ok(Item { app, api }) } else { Err(app) })
}

fn text(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(str::trim).filter(|text| !text.is_empty()).map(str::to_owned)
}

fn describe(app: &App, body: &Value, url: &Url) -> Card {
    let mut card = Card::host(app.key, url);
    card.title = text(body, "title").unwrap_or_else(|| "Sans titre".to_owned());
    card.status = text(body, "status");
    card.subtitle = text(body, "start_at");
    card
}

async fn fetch(state: &AppState, user_id: i64, item: &Item<'_>) -> Option<Value> {
    let endpoint = format!("{}{}", item.app.url, item.api);
    let build = |http: &reqwest::Client, token: &str| http.get(&endpoint).bearer_auth(token);
    let response = state.suite.send(user_id, item.app, false, build).await.ok()?;
    if !response.status().is_success() {
        return None;
    }
    response.json().await.ok()
}

/// The card of a link into a suite app, read with `user_id`'s own token, or `None` for a link
/// outside the suite. A Nuage link, an unknown page or an item the person cannot open is a
/// card with only the host.
pub async fn resolve(state: &AppState, user_id: i64, url: &Url) -> Option<Card> {
    let item = match item(state, url)? {
        Ok(item) if item.app.key != "nuage" => item,
        Ok(item) => return Some(Card::host(item.app.key, url)),
        Err(app) => return Some(Card::host(app.key, url)),
    };
    Some(match fetch(state, user_id, &item).await {
        Some(body) => describe(item.app, &body, url),
        None => Card::host(item.app.key, url),
    })
}
