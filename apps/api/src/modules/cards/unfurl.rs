use url::Url;

use super::Card;
use super::guard::allowed;
use super::og;

/// The most of a page an unfurl reads.
pub const MAX_BYTES: usize = 1 << 20;

async fn body(response: reqwest::Response) -> Option<String> {
    let html = response
        .headers()
        .get(http::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.to_ascii_lowercase().starts_with("text/html"));
    if !html || !response.status().is_success() {
        return None;
    }
    let mut response = response;
    let mut bytes = Vec::new();
    while let Ok(Some(chunk)) = response.chunk().await {
        let room = MAX_BYTES.saturating_sub(bytes.len());
        bytes.extend_from_slice(&chunk[..chunk.len().min(room)]);
        if bytes.len() >= MAX_BYTES {
            break;
        }
    }
    Some(String::from_utf8_lossy(&bytes).into_owned())
}

/// A link card from the page's Open Graph tags, fetched through the guarded client; the bare
/// host when the URL may not be fetched or the page says nothing.
pub async fn unfurl(http: &reqwest::Client, url: &Url) -> Card {
    let fallback = Card::link(url);
    if !allowed(url) {
        return fallback;
    }
    let Ok(response) = http.get(url.as_str()).send().await else { return fallback };
    let Some(html) = body(response).await else { return fallback };
    let meta = og::parse(&html);
    Card { title: meta.title.unwrap_or(fallback.title), subtitle: meta.description.or(meta.site), ..fallback }
}
