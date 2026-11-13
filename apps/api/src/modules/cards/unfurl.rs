use url::Url;

use super::Card;
use super::guard::allowed;
use super::og;

/// The most of a page an unfurl reads.
pub const MAX_BYTES: usize = 1 << 20;

fn video_id(raw: &str) -> Option<String> {
    let valid = raw.len() == 11 && raw.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_');
    valid.then(|| raw.to_owned())
}

/// The id of the YouTube video `url` points at: `youtu.be/<id>`, `watch?v=<id>`, `/shorts/`,
/// `/embed/`, `/live/` or `/v/` on youtube.com, its subdomains and youtube-nocookie.com.
fn youtube_id(url: &Url) -> Option<String> {
    let host = url.host_str()?.trim_start_matches("www.").trim_start_matches("m.");
    let parts: Vec<&str> = url.path_segments().map(|p| p.filter(|s| !s.is_empty()).collect()).unwrap_or_default();
    if host == "youtu.be" {
        return parts.first().and_then(|id| video_id(id));
    }
    let yt = host == "youtube.com" || host.ends_with(".youtube.com") || host == "youtube-nocookie.com";
    if !yt {
        return None;
    }
    match parts.as_slice() {
        ["watch", ..] => url.query_pairs().find(|(k, _)| k == "v").and_then(|(_, v)| video_id(&v)),
        ["shorts" | "embed" | "live" | "v", id, ..] => video_id(id),
        _ => None,
    }
}

fn youtube_card(url: &Url, id: &str) -> Card {
    let host = url.host_str().unwrap_or(url.as_str());
    Card {
        kind: "youtube",
        title: host.to_owned(),
        subtitle: None,
        status: None,
        icon: "youtube",
        url: url.to_string(),
        image: Some(format!("https://img.youtube.com/vi/{id}/hqdefault.jpg")),
        favicon: Some(format!("https://www.google.com/s2/favicons?domain={host}&sz=64")),
        site: Some("YouTube".to_owned()),
    }
}

fn resolve_url(base: &Url, raw: Option<String>) -> Option<String> {
    let rel = raw?.trim().to_owned();
    if rel.is_empty() {
        return None;
    }
    Some(base.join(&rel).map(|u| u.to_string()).unwrap_or(rel))
}

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
    if let Some(id) = youtube_id(url) {
        return youtube_card(url, &id);
    }
    let fallback = Card::link(url);
    if !allowed(url) {
        return fallback;
    }
    let Ok(response) = http.get(url.as_str()).send().await else { return fallback };
    let Some(html) = body(response).await else { return fallback };
    let meta = og::parse(&html);
    let image = resolve_url(url, meta.image);
    let favicon = resolve_url(url, meta.favicon)
        .or_else(|| url.host_str().map(|h| format!("https://www.google.com/s2/favicons?domain={h}&sz=64")));
    let site = meta.site;
    Card {
        title: meta.title.unwrap_or(fallback.title),
        subtitle: meta.description.or_else(|| site.clone()),
        image,
        favicon,
        site,
        ..fallback
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn youtube_ids_survive_share_params_and_refuse_junk() {
        let cases = [
            ("https://youtu.be/PAy5EBTnV6Q?list=RDERCkf0j8pPM", Some("PAy5EBTnV6Q")),
            ("https://youtu.be/PAy5EBTnV6Q?t=42", Some("PAy5EBTnV6Q")),
            ("https://www.youtube.com/watch?v=PAy5EBTnV6Q&list=RDERCkf0j8pPM&index=2", Some("PAy5EBTnV6Q")),
            ("https://m.youtube.com/watch?feature=share&v=PAy5EBTnV6Q&t=1m2s", Some("PAy5EBTnV6Q")),
            ("https://music.youtube.com/watch?v=PAy5EBTnV6Q", Some("PAy5EBTnV6Q")),
            ("https://www.youtube.com/live/PAy5EBTnV6Q?si=abc", Some("PAy5EBTnV6Q")),
            ("https://www.youtube-nocookie.com/embed/PAy5EBTnV6Q", Some("PAy5EBTnV6Q")),
            ("https://youtu.be/", None),
            ("https://youtu.be/../../admin", None),
            ("https://www.youtube.com/watch?v=short", None),
            ("https://notyoutube.com/watch?v=PAy5EBTnV6Q", None),
        ];
        for (raw, id) in cases {
            assert_eq!(youtube_id(&Url::parse(raw).unwrap()).as_deref(), id, "{raw}");
        }
    }

    #[test]
    fn youtube_and_relative_urls() {
        let watch = Url::parse("https://www.youtube.com/watch?v=dQw4w9WgXcQ").unwrap();
        assert_eq!(youtube_id(&watch).as_deref(), Some("dQw4w9WgXcQ"));
        let short = Url::parse("https://youtu.be/dQw4w9WgXcQ").unwrap();
        assert_eq!(youtube_id(&short).as_deref(), Some("dQw4w9WgXcQ"));
        let shorts = Url::parse("https://www.youtube.com/shorts/dQw4w9WgXcQ").unwrap();
        assert_eq!(youtube_id(&shorts).as_deref(), Some("dQw4w9WgXcQ"));
        let embed = Url::parse("https://youtube.com/embed/dQw4w9WgXcQ").unwrap();
        assert_eq!(youtube_id(&embed).as_deref(), Some("dQw4w9WgXcQ"));
        let other = Url::parse("https://youtube.com/feed/explore").unwrap();
        assert_eq!(youtube_id(&other), None);

        let base = Url::parse("https://example.com/blog/article").unwrap();
        assert_eq!(
            resolve_url(&base, Some("/img/pic.png".to_string())),
            Some("https://example.com/img/pic.png".to_string())
        );
        assert_eq!(
            resolve_url(&base, Some("https://cdn.example.com/pic.png".to_string())),
            Some("https://cdn.example.com/pic.png".to_string())
        );
        assert_eq!(resolve_url(&base, None), None);
    }
}
