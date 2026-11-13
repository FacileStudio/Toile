use std::time::Duration;

use axum::response::{IntoResponse, Response};
use http::Method;
use porte::exchange::{Exchanger, Target};
use serde::Deserialize;
use serde_json::{Value, json};
use tronc::bulle::{self, ForgetFuture, Request, TokenFuture, TokenSource};
use uuid::Uuid;

use crate::extract::db;
use crate::modules::boards::access::{self, Access};
use crate::modules::suite::{self, AI_ACCESS};
use crate::state::AppState;

/// How long Toile waits for one completion.
pub const TIMEOUT: Duration = Duration::from_secs(90);

/// The largest board text sent to Bulle, under Bulle's 64 KiB input cap.
pub const MAX_INPUT: usize = 60 << 10;

const PLAN_MESSAGE: &str = "AI features need a paid plan.";

/// Why an AI call answers no result: the plan refusal muse's `planLimitFrom` reads, or any
/// other suite error.
#[derive(Debug)]
pub enum Refusal {
    /// `402` with `key: ai.access` and the upgrade link.
    Plan {
        /// The message shown to the person.
        message: String,
        /// Where they upgrade.
        upgrade_url: String,
    },
    /// Any other refusal, answered as the suite envelope.
    Error(tronc::Error),
}

impl From<tronc::Error> for Refusal {
    fn from(error: tronc::Error) -> Self {
        Refusal::Error(error)
    }
}

impl From<bulle::Error> for Refusal {
    fn from(error: bulle::Error) -> Self {
        match error {
            bulle::Error::PlanRequired(plan) => {
                let message = if plan.message.is_empty() { PLAN_MESSAGE.to_owned() } else { plan.message };
                Refusal::Plan { message, upgrade_url: plan.upgrade_url }
            }
            other => {
                tracing::warn!(error = %other, "ai: bulle refused or failed");
                Refusal::Error(other.into())
            }
        }
    }
}

impl IntoResponse for Refusal {
    fn into_response(self) -> Response {
        match self {
            Refusal::Plan { message, upgrade_url } => {
                let mut body = tronc::errors::envelope("payment_required", &message, Some(&upgrade_url));
                body["error"]["key"] = AI_ACCESS.into();
                (http::StatusCode::PAYMENT_REQUIRED, axum::Json(body)).into_response()
            }
            Refusal::Error(error) => error.into_response(),
        }
    }
}

/// A token source that exchanges the caller's Registre session for a Bulle token.
pub(super) struct Tokens<'a> {
    exchanger: &'a Exchanger,
    user_id: i64,
    target: Target,
}

impl TokenSource for Tokens<'_> {
    fn token(&self) -> TokenFuture<'_> {
        Box::pin(async move { Ok(self.exchanger.token(self.user_id, &self.target).await?) })
    }

    fn forget(&self) -> Option<ForgetFuture<'_>> {
        self.exchanger.forget(self.user_id, &self.target);
        Some(Box::pin(std::future::ready(())))
    }
}

/// Refuses unless `user_id` may edit `board` and its plan has `ai.access`: `404`/`403` from
/// the board, then `402` with the upgrade link. Nothing reaches Bulle before it passes.
pub async fn gate(state: &AppState, user_id: i64, board: Uuid) -> Result<(), Refusal> {
    access::check(state, user_id, board, Access::Editor).await?;
    let space_id: Option<String> = sqlx::query_scalar("SELECT space_id FROM boards WHERE id = $1")
        .bind(board)
        .fetch_one(&state.pool)
        .await
        .map_err(db)?;
    let scope = suite::scope(state, user_id, space_id.as_deref()).await?;
    if state.suite.plans.enabled(&scope, AI_ACCESS) {
        return Ok(());
    }
    let upgrade_url = porte::forfait::upgrade_url(&state.issuer, space_id.as_deref().unwrap_or(""));
    Err(Refusal::Plan { message: PLAN_MESSAGE.to_owned(), upgrade_url })
}

/// One completion: `prompt` is Toile's, `input` the board's untrusted text.
pub struct Ask {
    /// The instructions.
    pub prompt: String,
    /// The board content the instructions work on.
    pub input: String,
    /// A JSON Schema the answer must match, for a JSON answer.
    pub schema: Option<Value>,
    /// The output budget.
    pub max_tokens: u32,
}

#[derive(Deserialize)]
struct Completed {
    text: Option<String>,
    json: Option<Value>,
}

/// Cuts `text` to at most `max` bytes on a character boundary.
pub fn clip(text: &str, max: usize) -> &str {
    let mut end = text.len().min(max);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    &text[..end]
}

/// The Bulle client and the caller's token source, or `503` while Bulle is not connected.
pub(super) fn connect(state: &AppState, user_id: i64) -> Result<(bulle::Client, Tokens<'_>), Refusal> {
    let app = &state.suite.apps.bulle;
    let Some(exchanger) = state.suite.exchanger.as_ref().filter(|_| app.enabled()) else {
        return Err(unavailable().into());
    };
    let http = reqwest::Client::builder()
        .timeout(TIMEOUT)
        .redirect(reqwest::redirect::Policy::none())
        .no_proxy()
        .build()
        .map_err(|error| tronc::Error::internal("ai http client", error))?;
    let client = bulle::Client::with_http(&app.url, http).map_err(|_| unavailable())?;
    Ok((client, Tokens { exchanger, user_id, target: app.target(true) }))
}

fn unavailable() -> tronc::Error {
    tronc::Error::unavailable("AI is not connected to Toile")
}

/// Sends `ask` to Bulle's `POST /api/ai/complete` with a token exchanged for `user_id`, and
/// answers its text, or its JSON when `ask` has a schema.
pub async fn complete(state: &AppState, user_id: i64, ask: Ask) -> Result<Value, Refusal> {
    let (client, tokens) = connect(state, user_id)?;
    let mut body = json!({"prompt": ask.prompt, "input": clip(&ask.input, MAX_INPUT), "max_tokens": ask.max_tokens});
    if let Some(schema) = ask.schema {
        body["schema"] = schema;
    }
    let request = Request::new(Method::POST, ["ai", "complete"]).json(body);
    let done: Completed = client.call(&tokens, request).await?;
    match (done.json, done.text) {
        (Some(value), _) => Ok(value),
        (None, Some(text)) => Ok(Value::String(text)),
        (None, None) => Err(tronc::Error::unavailable("Bulle answered nothing").into()),
    }
}
