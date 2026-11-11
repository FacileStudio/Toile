use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tronc::mcp::{Answer, Call, RESULT_BUDGET, ToolError, clip};

use super::{board, user};
use crate::modules::boards::service;
use crate::modules::boards::types::Board;
use crate::modules::spaces;
use crate::scene;
use crate::state::AppState;

/// `toile_list_boards`.
#[derive(Deserialize, JsonSchema)]
pub struct ListIn {
    /// One space's boards only. Absent: your personal boards and every space's.
    pub space_id: Option<String>,
}

/// One board.
#[derive(Serialize, JsonSchema)]
pub struct Listed {
    pub id: String,
    pub name: String,
    /// The space; absent for a personal board.
    pub space_id: Option<String>,
    /// owner, editor, commenter or viewer.
    pub role: String,
    pub updated_at: String,
    pub url: String,
}

/// One of your spaces.
#[derive(Serialize, JsonSchema)]
pub struct SpaceOut {
    pub id: String,
    pub name: String,
}

/// What `toile_list_boards` answers, most recently changed first per scope.
#[derive(Serialize, JsonSchema)]
pub struct ListOut {
    pub boards: Vec<Listed>,
    pub spaces: Vec<SpaceOut>,
}

fn listed(board: Board, origin: &str) -> Listed {
    let updated_at = board.updated_at.format(&time::format_description::well_known::Rfc3339).unwrap_or_default();
    Listed {
        url: format!("{origin}/b/{}", board.id),
        id: board.id.to_string(),
        name: board.name,
        space_id: board.space_id,
        role: board.role.to_owned(),
        updated_at,
    }
}

/// `toile_list_boards`: the boards of one scope, or of every scope the caller is in.
pub async fn list(state: AppState, origin: String, call: Call, input: ListIn) -> Result<Answer<ListOut>, ToolError> {
    let me = user(&call)?;
    let spaces = spaces::mine(&state, me).await?;
    let scopes: Vec<Option<String>> = match input.space_id {
        Some(space) => vec![Some(space)],
        None => std::iter::once(None).chain(spaces.iter().map(|s| Some(s.id.clone()))).collect(),
    };
    let mut boards = Vec::new();
    for scope in scopes {
        boards
            .extend(service::list(&state, me, scope.as_deref(), false).await?.into_iter().map(|b| listed(b, &origin)));
    }
    let spaces = spaces.into_iter().map(|s| SpaceOut { id: s.id, name: s.name }).collect();
    Ok(ListOut { boards, spaces }.into())
}

/// `toile_read_board`.
#[derive(Deserialize, JsonSchema)]
pub struct ReadIn {
    /// The board id, from `toile_list_boards`.
    pub board_id: String,
}

/// What `toile_read_board` answers.
#[derive(Serialize, JsonSchema)]
pub struct ReadOut {
    pub id: String,
    pub name: String,
    /// owner, editor, commenter or viewer: editors and owners may change shapes.
    pub role: String,
    pub shape_count: usize,
    /// One line per shape (`type id at x,y wxh color: "text"`), each frame followed by its
    /// children, then `outside frames:` and the rest. Arrows name the shapes they join.
    pub shapes: String,
    /// True when `shapes` was cut to fit the answer.
    pub truncated: bool,
    pub url: String,
    /// The board as SVG, for a picture of it (needs your Toile session).
    pub svg_url: String,
    pub thumbnail_url: Option<String>,
}

/// `toile_read_board`: every shape as compact text, grouped by frame.
pub async fn read(state: AppState, origin: String, call: Call, input: ReadIn) -> Result<Answer<ReadOut>, ToolError> {
    let id = board(&input.board_id)?;
    let found = service::read(&state, user(&call)?, id).await?;
    let shapes = scene::load(&state, id).await?;
    let text = scene::text::render(&shapes);
    let (text, truncated) = clip(&text, RESULT_BUDGET);
    Ok(ReadOut {
        id: id.to_string(),
        name: found.name,
        role: found.role.to_owned(),
        shape_count: shapes.len(),
        shapes: text.to_owned(),
        truncated,
        url: format!("{origin}/b/{id}"),
        svg_url: format!("{origin}/api/boards/{id}/export.svg"),
        thumbnail_url: found.thumbnail_url.map(|path| format!("{origin}{path}")),
    }
    .into())
}
