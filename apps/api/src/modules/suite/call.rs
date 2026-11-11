use std::time::Duration;

use tronc::{Error, Result};

use super::Suite;
use super::apps::App;

/// How long a call to another suite app may take.
pub const TIMEOUT: Duration = Duration::from_secs(10);

/// The client for calls to the suite apps: no proxy from the environment, no redirects.
pub fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(TIMEOUT)
        .redirect(reqwest::redirect::Policy::none())
        .no_proxy()
        .build()
        .unwrap_or_default()
}

fn unreachable(app: &App, error: reqwest::Error) -> Error {
    tracing::warn!(app = app.key, error = %error, "suite: call failed");
    Error::unavailable(format!("{} is unreachable", app.key))
}

impl Suite {
    /// Sends the request `build` makes, with a token exchanged for `user_id` and `app`, retrying
    /// once with a fresh token when the app answers 401. `write` asks for the write scope too.
    /// The answer comes back whatever its status. A dead Registre grant is `412`; an app that
    /// is not configured, or Registre not configured, is `503`.
    pub async fn send<F>(&self, user_id: i64, app: &App, write: bool, build: F) -> Result<reqwest::Response>
    where
        F: Fn(&reqwest::Client, &str) -> reqwest::RequestBuilder,
    {
        let Some(exchanger) = self.exchanger.as_ref().filter(|_| app.enabled()) else {
            return Err(Error::unavailable(format!("{} is not connected to Toile", app.key)));
        };
        let target = app.target(write);
        let token = exchanger.token(user_id, &target).await?;
        let response = build(&self.http, &token).send().await.map_err(|error| unreachable(app, error))?;
        if response.status() != reqwest::StatusCode::UNAUTHORIZED {
            return Ok(response);
        }
        exchanger.forget(user_id, &target);
        let token = exchanger.token(user_id, &target).await?;
        build(&self.http, &token).send().await.map_err(|error| unreachable(app, error))
    }
}
