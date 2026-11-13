use schemars::JsonSchema;
use serde::Deserialize;
use tronc::mcp::{Answer, Call, ToolError, failure};

use super::boards::{Listed, listed};
use super::{board, user};
use crate::modules::boards::access::{self, Access};
use crate::modules::boards::relocate::relocate;
use crate::modules::boards::types::{Board, CreateRequest, NameRequest};
use crate::modules::boards::{service, trash};
use crate::state::AppState;

/// The `move_to` value that names the caller's personal boards.
pub const PERSONAL: &str = "personal";

/// `toile_create_board`.
#[derive(Deserialize, JsonSchema)]
pub struct CreateIn {
    /// The board name; blank gives "Sans titre".
    #[serde(default, alias = "title")]
    pub name: String,
    /// The space to create it in, from `toile_list_boards`. Absent: your personal boards.
    pub space_id: Option<String>,
}

/// `toile_create_board`: an empty board in your personal boards or a space you edit in.
pub async fn create(state: AppState, origin: String, call: Call, input: CreateIn) -> Result<Answer<Listed>, ToolError> {
    let request = CreateRequest { name: input.name, space_id: input.space_id };
    Ok(listed(service::create(&state, user(&call)?, request).await?, &origin).into())
}

/// `toile_update_board`.
#[derive(Deserialize, JsonSchema)]
pub struct UpdateIn {
    /// The board id.
    pub board_id: String,
    /// A new name.
    pub name: Option<String>,
    /// A space id to move the board to, or `personal` for your personal boards. Needs owner.
    pub move_to: Option<String>,
}

/// `toile_update_board`: renames a board, moves it to another scope, or both.
pub async fn update(state: AppState, origin: String, call: Call, input: UpdateIn) -> Result<Answer<Listed>, ToolError> {
    let (id, me) = (board(&input.board_id)?, user(&call)?);
    if input.name.is_none() && input.move_to.is_none() {
        return Err(ToolError("give name, move_to, or both".to_owned()));
    }
    let name = input.name.as_deref().map(service::name).transpose()?;
    if let Some(to) = input.move_to.as_deref().map(str::trim) {
        let hint = "move_to takes a space id from toile_list_boards, or personal";
        relocate(&state, me, id, Some(to).filter(|to| *to != PERSONAL)).await.map_err(|e| failure(e, hint))?;
    }
    Ok(listed(service::rename(&state, me, id, NameRequest { name }).await?, &origin).into())
}

/// `toile_trash_board` and `toile_restore_board`.
#[derive(Deserialize, JsonSchema)]
pub struct BoardIn {
    /// The board id.
    pub board_id: String,
}

/// `toile_trash_board`: moves a board to the trash, where it stays 30 days; a board already
/// there answers the same.
pub async fn trash(state: AppState, origin: String, call: Call, input: BoardIn) -> Result<Answer<Listed>, ToolError> {
    let (id, me) = (board(&input.board_id)?, user(&call)?);
    if access::lookup(&state, me, id, true).await? != Access::Owner {
        trash::trash(&state, me, id).await?;
    }
    let row = service::fetch(&state, id).await?;
    Ok(listed(Board::from_row(row, Access::Owner), &origin).into())
}

/// `toile_restore_board`: takes a board out of the trash; a live board answers as it is.
pub async fn restore(state: AppState, origin: String, call: Call, input: BoardIn) -> Result<Answer<Listed>, ToolError> {
    let (id, me) = (board(&input.board_id)?, user(&call)?);
    let found = match access::access(&state, me, id).await? {
        Access::None => trash::restore(&state, me, id).await?,
        _ => service::read(&state, me, id).await?,
    };
    Ok(listed(found, &origin).into())
}
