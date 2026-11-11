use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::Card;

/// How long a resolved card is reused for the same person and URL.
pub const TTL: Duration = Duration::from_secs(600);
const MAX_ENTRIES: usize = 10_000;

type Entries = HashMap<(i64, String), (Instant, Card)>;

/// Resolved cards per (person, URL) for ten minutes, and the guarded unfurl client.
#[derive(Clone)]
pub struct Cards {
    entries: Arc<Mutex<Entries>>,
    /// The SSRF-guarded client for Open Graph unfurls.
    pub unfurl: reqwest::Client,
}

impl Default for Cards {
    fn default() -> Self {
        Cards { entries: Arc::default(), unfurl: super::guard::client() }
    }
}

impl Cards {
    fn lock(&self) -> std::sync::MutexGuard<'_, Entries> {
        self.entries.lock().unwrap_or_else(|poison| poison.into_inner())
    }

    /// The card cached for `user_id` and `url`, while it is fresh.
    pub fn get(&self, user_id: i64, url: &str) -> Option<Card> {
        let entries = self.lock();
        let (at, card) = entries.get(&(user_id, url.to_owned()))?;
        (at.elapsed() < TTL).then(|| card.clone())
    }

    /// Keeps `card` for `user_id` and `url`; past 10,000 entries the stale ones go first.
    pub fn put(&self, user_id: i64, url: &str, card: Card) {
        let mut entries = self.lock();
        if entries.len() >= MAX_ENTRIES {
            entries.retain(|_, (at, _)| at.elapsed() < TTL);
        }
        if entries.len() >= MAX_ENTRIES {
            entries.clear();
        }
        entries.insert((user_id, url.to_owned()), (Instant::now(), card));
    }
}
