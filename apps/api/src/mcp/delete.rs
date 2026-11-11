use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tronc::mcp::{Action, Answer, Call, Confirmations, Kind, ToolError};
use uuid::Uuid;

use super::{board, user};
use crate::modules::boards::access::{self, Access};
use crate::modules::boards::service;
use crate::scene::{self, text, write};
use crate::state::AppState;

/// The confirmation kind of `toile_delete_shapes`.
pub const KIND: &str = "toile_delete_shapes";

/// `toile_delete_shapes`, and what a pending action stores as its target.
#[derive(Deserialize, Serialize, JsonSchema)]
pub struct DeleteIn {
    /// The board id.
    pub board_id: String,
    /// The shapes to delete. A frame takes the shapes inside it along.
    pub ids: Vec<String>,
}

/// What `toile_delete_shapes` answers.
#[derive(Serialize, JsonSchema)]
pub struct DeleteOut {
    /// `done`, `declined` or `pending_confirmation`.
    pub status: String,
    /// The link the person opens to confirm, when the client could not ask.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub confirm_url: String,
}

fn target(action: &Action) -> tronc::Result<(i64, Uuid, DeleteIn)> {
    let input: DeleteIn =
        serde_json::from_str(&action.target).map_err(|e| tronc::Error::internal("bad action target", e))?;
    let user = action.user_id.parse().map_err(|e| tronc::Error::internal("bad action user", e))?;
    Ok((user, crate::extract::id(&input.board_id)?, input))
}

async fn doomed(state: &AppState, id: Uuid, ids: &[String]) -> tronc::Result<Vec<String>> {
    let all = scene::load(state, id).await?;
    let gone = write::with_children(&all, ids);
    Ok(all.iter().filter(|s| gone.contains(&s.id)).map(text::line).collect())
}

async fn fingerprint(state: AppState, action: Action) -> tronc::Result<String> {
    let (user, id, input) = target(&action)?;
    access::check(&state, user, id, Access::Editor).await?;
    Ok(doomed(&state, id, &input.ids).await?.join("\n"))
}

async fn run(state: AppState, action: Action) -> tronc::Result<()> {
    let (user, id, input) = target(&action)?;
    access::check(&state, user, id, Access::Editor).await?;
    let ids = input.ids;
    let edit =
        Box::new(move |txn: &mut yrs::TransactionMut, map: &yrs::MapRef| write::remove(txn, map, &ids).map(drop));
    state.hub.write(&state, id, edit).await
}

/// The confirmation kind that deletes the shapes once the person agrees.
pub fn kind(state: &AppState) -> Kind {
    let (a, b) = (state.clone(), state.clone());
    Kind::new(KIND, move |action| fingerprint(a.clone(), action), move |action| run(b.clone(), action))
}

/// `toile_delete_shapes`: asks the person, then deletes the shapes if they agree and the shapes
/// did not change meanwhile.
pub async fn delete(
    state: AppState,
    confirm: Confirmations,
    call: Call,
    input: DeleteIn,
) -> Result<Answer<DeleteOut>, ToolError> {
    if call.retrying() {
        let outcome = confirm.resume(&call, KIND).await?;
        return Ok(DeleteOut { status: outcome.status.to_owned(), confirm_url: outcome.confirm_url }.into());
    }
    let id = board(&input.board_id)?;
    access::check(&state, user(&call)?, id, Access::Editor).await?;
    let lines = doomed(&state, id, &input.ids).await?;
    if input.ids.is_empty() || lines.len() < input.ids.len() {
        return Err(ToolError("name shapes the board holds; call toile_read_board for their ids".to_owned()));
    }
    let name = service::fetch(&state, id).await?.name;
    let shown: Vec<&str> = lines.iter().take(20).map(String::as_str).collect();
    let question = format!("Delete {} shapes from \"{name}\"?\n{}", lines.len(), shown.join("\n"));
    let target = serde_json::to_string(&input).map_err(|e| ToolError(e.to_string()))?;
    let (ask, outcome) = confirm.request(&call, KIND, &target, &question).await?;
    match ask {
        Some(question) => Ok(Answer::InputRequired(question)),
        None => Ok(DeleteOut { status: outcome.status.to_owned(), confirm_url: outcome.confirm_url }.into()),
    }
}
