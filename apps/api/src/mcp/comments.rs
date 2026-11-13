use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use time::format_description::well_known::Rfc3339;
use tronc::mcp::{Answer, Call, ToolError, encoded_size, failure, fit_count};

use super::{board, page, user};
use crate::extract::id as parse_id;
use crate::modules::boards::access::{self, Access};
use crate::modules::comments::service;
use crate::modules::comments::types::{Comment, PatchRequest};
use crate::state::AppState;

/// Which threads `toile_list_comments` answers.
#[derive(Deserialize, JsonSchema, Default, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    /// Threads nobody resolved yet.
    #[default]
    Open,
    /// Resolved threads.
    Resolved,
    /// Both.
    All,
}

/// `toile_list_comments`.
#[derive(Deserialize, JsonSchema)]
pub struct ListIn {
    /// The board id.
    pub board_id: String,
    /// open (the default), resolved or all.
    #[serde(default)]
    pub status: Status,
    /// The most threads to answer, 50 by default, 200 at most.
    pub limit: Option<usize>,
    /// The `next_cursor` of the previous page.
    pub cursor: Option<String>,
}

/// One comment or reply.
#[derive(Serialize, JsonSchema)]
pub struct Note {
    pub id: String,
    pub author: String,
    pub body: String,
    pub created_at: String,
}

/// A thread: its first comment, where it is pinned, and its replies, oldest first.
#[derive(Serialize, JsonSchema)]
pub struct Thread {
    #[serde(flatten)]
    pub comment: Note,
    /// The shape it is pinned on, when it is.
    pub shape_id: Option<String>,
    pub x: f64,
    pub y: f64,
    pub resolved: bool,
    pub replies: Vec<Note>,
    /// True when the thread has more replies than fit in one answer; the oldest ones are listed.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub replies_truncated: bool,
}

/// What `toile_list_comments` answers, oldest thread first.
#[derive(Serialize, JsonSchema)]
pub struct ListOut {
    pub threads: Vec<Thread>,
    /// Pass it as `cursor` for the next threads; absent on the last page.
    pub next_cursor: Option<String>,
}

fn note(comment: &Comment) -> Note {
    Note {
        id: comment.id.to_string(),
        author: comment.author.clone(),
        body: comment.body.clone(),
        created_at: comment.created_at.format(&Rfc3339).unwrap_or_default(),
    }
}

fn thread(top: &Comment, all: &[Comment]) -> Thread {
    let mut replies: Vec<Note> = all.iter().filter(|c| c.parent == Some(top.id)).map(note).collect();
    let anchor = &top.anchor.0;
    let mut thread = Thread {
        comment: note(top),
        shape_id: anchor.shape.clone(),
        x: anchor.x,
        y: anchor.y,
        resolved: top.resolved_at.is_some(),
        replies: Vec::new(),
        replies_truncated: false,
    };
    let base = encoded_size(&thread);
    let fitted = fit_count(replies.len(), |n| base + encoded_size(&replies[..n]));
    thread.replies_truncated = fitted < replies.len();
    replies.truncate(fitted);
    thread.replies = replies;
    thread
}

/// `toile_list_comments`: the threads of a board with their replies, a page at a time.
pub async fn list(state: AppState, call: Call, input: ListIn) -> Result<Answer<ListOut>, ToolError> {
    let (id, me) = (board(&input.board_id)?, user(&call)?);
    access::check(&state, me, id, Access::Viewer).await?;
    let all = service::list(&state, id, Some(me)).await?;
    let wanted = |c: &&Comment| match input.status {
        Status::Open => c.resolved_at.is_none(),
        Status::Resolved => c.resolved_at.is_some(),
        Status::All => true,
    };
    let threads: Vec<Thread> =
        all.iter().filter(|c| c.parent.is_none()).filter(wanted).map(|c| thread(c, &all)).collect();
    let (threads, next_cursor) = page(threads, input.cursor.as_deref(), input.limit)?;
    Ok(ListOut { threads, next_cursor }.into())
}

/// `toile_resolve_comment`.
#[derive(Deserialize, JsonSchema)]
pub struct ResolveIn {
    /// The board id.
    pub board_id: String,
    /// The thread id, from `toile_list_comments`.
    pub comment_id: String,
    /// False reopens a resolved thread. Default true.
    pub resolved: Option<bool>,
}

/// What `toile_resolve_comment` answers.
#[derive(Serialize, JsonSchema)]
pub struct ResolveOut {
    pub id: String,
    pub resolved: bool,
}

/// `toile_resolve_comment`: resolves or reopens a thread; everyone on the board sees it live.
pub async fn resolve(state: AppState, call: Call, input: ResolveIn) -> Result<Answer<ResolveOut>, ToolError> {
    let (id, me) = (board(&input.board_id)?, user(&call)?);
    let comment = parse_id(&input.comment_id)
        .map_err(|_| ToolError(format!("{:?} is not a comment id; call toile_list_comments", input.comment_id)))?;
    let patch = PatchRequest { body: None, resolved: Some(input.resolved.unwrap_or(true)) };
    let hint = "call toile_list_comments for thread ids";
    let done = service::update(&state, me, id, comment, patch).await.map_err(|e| failure(e, hint))?;
    Ok(ResolveOut { id: done.id.to_string(), resolved: done.resolved_at.is_some() }.into())
}
