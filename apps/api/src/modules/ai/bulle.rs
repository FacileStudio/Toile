use std::time::Duration;

use porte::exchange::Bound;
use porte::forfait::Scope;
use serde_json::Value;
use tronc::bulle::{self, CompleteRequest};
use tronc::errors::Code;
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

/// The suite envelope for a Bulle failure. Bulle's 402 names no entitlement, so it gets
/// `key: ai.access`, which muse's `planLimitFrom` reads.
pub fn refused(error: bulle::Error) -> tronc::Error {
    if error.status() != Some(402) {
        tracing::warn!(error = %error, "ai: bulle refused or failed");
    }
    let error = tronc::Error::from(error);
    match (error.code, &error.key) {
        (Code::PaymentRequired, None) => error.with_key(AI_ACCESS),
        _ => error,
    }
}

async fn board_space(state: &AppState, board: Uuid) -> tronc::Result<Option<String>> {
    sqlx::query_scalar("SELECT space_id FROM boards WHERE id = $1").bind(board).fetch_one(&state.pool).await.map_err(db)
}

/// Refuses unless `user_id` may edit `board` and has `ai.access` on their own plan or on the
/// board's space plan, as Bulle grants it: `404`/`403` from the board, then `402` with the
/// upgrade link. Nothing reaches Bulle before it passes.
pub async fn gate(state: &AppState, user_id: i64, board: Uuid) -> tronc::Result<()> {
    access::check(state, user_id, board, Access::Editor).await?;
    let space_id = board_space(state, board).await?;
    let plans = &state.suite.plans;
    if let Some(space) = space_id.as_deref()
        && plans.enabled(&Scope::space(space), AI_ACCESS)
    {
        return Ok(());
    }
    if plans.enabled(&suite::scope(state, user_id, None).await?, AI_ACCESS) {
        return Ok(());
    }
    let upgrade_url = porte::forfait::upgrade_url(&state.issuer, space_id.as_deref().unwrap_or(""));
    Err(tronc::Error::payment_required(PLAN_MESSAGE).with_upgrade_url(upgrade_url).with_key(AI_ACCESS))
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

/// The pooled Bulle client and the caller's token source, or `503` while Bulle is not
/// connected.
pub(super) fn connect(state: &AppState, user_id: i64) -> tronc::Result<(&bulle::Client, Bound)> {
    let app = &state.suite.apps.bulle;
    let (Some(client), Some(exchanger)) = (state.suite.bulle.as_ref(), state.suite.exchanger.as_ref()) else {
        return Err(tronc::Error::unavailable("AI is not connected to Toile"));
    };
    Ok((client, exchanger.bound(user_id, app.target(true))))
}

/// Sends `ask` about `board` to Bulle's `POST /api/ai/complete` with a token exchanged for
/// `user_id`, naming the board's space when it has one, and answers its text, or its JSON when
/// `ask` has a schema.
pub async fn complete(state: &AppState, user_id: i64, board: Uuid, ask: Ask) -> tronc::Result<Value> {
    let (client, tokens) = connect(state, user_id)?;
    let space_id = board_space(state, board).await?.unwrap_or_default();
    let request = CompleteRequest {
        prompt: ask.prompt,
        input: tronc::mcp::clip(&ask.input, MAX_INPUT).0.to_owned(),
        schema: ask.schema,
        max_tokens: ask.max_tokens,
        space_id,
        ..CompleteRequest::default()
    };
    let done = client.complete(&tokens, &request).await.map_err(refused)?;
    match (done.json, done.text) {
        (Some(value), _) => Ok(value),
        (None, text) if !text.is_empty() => Ok(Value::String(text)),
        (None, _) => Err(tronc::Error::unavailable("Bulle answered nothing")),
    }
}
