use axum::Router;
use axum::extract::State;
use axum::routing::post;
use porte::Identity;
use serde::Deserialize;
use tronc::json::Json;
use tronc::{Error, Result};
use url::Url;

use super::{Card, apps, unfurl};
use crate::state::AppState;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Resolve {
    url: String,
}

fn parse(raw: &str) -> Result<Url> {
    let url = Url::parse(raw.trim()).map_err(|_| Error::invalid("url is not a valid link"))?;
    match (url.scheme(), raw.len() <= 2048) {
        ("http" | "https", true) => Ok(url),
        _ => Err(Error::invalid("url must be an http or https link of at most 2048 characters")),
    }
}

async fn resolve(State(state): State<AppState>, who: Identity, Json(body): Json<Resolve>) -> Result<Json<Card>> {
    let url = parse(&body.url)?;
    let cards = &state.suite.cards;
    if let Some(card) = cards.get(who.user_id, url.as_str()) {
        return Ok(Json(card));
    }
    let card = match apps::resolve(&state, who.user_id, &url).await {
        Some(card) => card,
        None => unfurl::unfurl(&cards.unfurl, &url).await,
    };
    cards.put(who.user_id, url.as_str(), card.clone());
    Ok(Json(card))
}

/// `POST /cards/resolve`, behind `require_auth`.
pub fn routes() -> Router<AppState> {
    Router::new().route("/cards/resolve", post(resolve))
}
