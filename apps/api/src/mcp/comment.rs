use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tronc::mcp::{Answer, Call, ToolError};

use super::{board, user};
use crate::extract::id as parse_id;
use crate::modules::boards::access::{self, Access};
use crate::modules::comments::service::{self, Author};
use crate::modules::comments::types::{Anchor, CreateRequest};
use crate::scene;
use crate::state::AppState;

/// `toile_comment`.
#[derive(Deserialize, JsonSchema)]
pub struct CommentIn {
    /// The board id.
    pub board_id: String,
    /// The comment text.
    pub body: String,
    /// Pin it on this shape (its top-right corner).
    pub shape_id: Option<String>,
    /// Or pin it at this board point.
    pub x: Option<f64>,
    pub y: Option<f64>,
    /// Reply in this thread (a top-level comment id) instead.
    pub reply_to: Option<String>,
}

/// What `toile_comment` answers.
#[derive(Serialize, JsonSchema)]
pub struct CommentOut {
    pub id: String,
    pub author: String,
}

async fn anchor(state: &AppState, board: uuid::Uuid, input: &CommentIn) -> Result<Option<Anchor>, ToolError> {
    if input.reply_to.is_some() {
        return Ok(None);
    }
    let Some(shape) = &input.shape_id else {
        let (x, y) = input.x.zip(input.y).ok_or_else(|| ToolError("give shape_id, x and y, or reply_to".into()))?;
        return Ok(Some(Anchor { shape: None, x, y }));
    };
    let found = scene::load(state, board).await?.into_iter().find(|s| &s.id == shape);
    let found = found.ok_or_else(|| ToolError(format!("no shape {shape:?} on the board")))?;
    Ok(Some(Anchor { shape: Some(found.id), x: found.w, y: 0.0 }))
}

/// `toile_comment`: a comment pinned on a shape or a point, or a reply, signed by the caller.
pub async fn comment(state: AppState, call: Call, input: CommentIn) -> Result<Answer<CommentOut>, ToolError> {
    let (id, me) = (board(&input.board_id)?, user(&call)?);
    access::check(&state, me, id, Access::Commenter).await?;
    let anchor = anchor(&state, id, &input).await?;
    let parent = input.reply_to.as_deref().map(parse_id).transpose()?;
    let request = CreateRequest { anchor, body: input.body, parent };
    let made = service::create(&state, id, Author::User(me), request).await?;
    Ok(CommentOut { id: made.id.to_string(), author: made.author }.into())
}
