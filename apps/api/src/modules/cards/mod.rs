//! Live cards: a pasted link resolved into a title, a status and an icon, read from the suite
//! app it points at with the viewing person's own token, or unfurled from Open Graph tags.

mod apps;
mod cache;
mod guard;
#[cfg(test)]
mod guard_tests;
mod og;
mod router;
mod unfurl;

use serde::Serialize;
use url::Url;

pub use cache::Cards;
pub use router::routes;

/// One resolved link, as `POST /cards/resolve` answers it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Card {
    /// `docs`, `feuilles`, `opus`, `agenda`, `nuage` or `link`.
    pub kind: &'static str,
    /// The item's title, or the host when nothing better is known.
    pub title: String,
    /// A second line: a description, a date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
    /// The item's status, such as an Opus task's.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// The icon to show: the app's key, or `link`.
    pub icon: &'static str,
    /// The link itself.
    pub url: String,
    /// A preview image, when the unfurled page provides one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    /// The site's favicon URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favicon: Option<String>,
    /// The name of the website or publication.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub site: Option<String>,
}

impl Card {
    /// A card that only names the host of `url`, under `kind`.
    pub fn host(kind: &'static str, url: &Url) -> Self {
        let title = url.host_str().unwrap_or(url.as_str()).to_owned();
        Card {
            kind,
            title,
            subtitle: None,
            status: None,
            icon: kind,
            url: url.to_string(),
            image: None,
            favicon: None,
            site: None,
        }
    }

    /// [`Card::host`] for a link outside the suite.
    pub fn link(url: &Url) -> Self {
        Card::host("link", url)
    }
}
