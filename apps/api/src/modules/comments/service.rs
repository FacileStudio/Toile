use tronc::{Error, Result};
use uuid::Uuid;

use super::types::{Anchor, Comment, CreateRequest, PatchRequest, columns, text};
use crate::extract::db;
use crate::modules::boards::access::{self, Access};
use crate::modules::rooms::{Command, comments_changed};
use crate::state::AppState;

/// Who writes a comment.
#[derive(Debug, Clone)]
pub enum Author {
    /// A signed-in person.
    User(i64),
    /// A share-link guest: display name and link.
    Guest(String, Uuid),
}

/// Tells the board's open clients to refetch its comments.
pub async fn notify(state: &AppState, board: Uuid) {
    state.hub.tell(board, Command::Broadcast(comments_changed())).await;
}

/// Every comment of `board`, oldest first; `me` marks the caller's own.
pub async fn list(state: &AppState, board: Uuid, me: Option<i64>) -> Result<Vec<Comment>> {
    sqlx::query_as::<_, Comment>(concat!("SELECT ", columns!(), " WHERE c.board_id = $1 ORDER BY c.created_at, c.id"))
        .bind(board)
        .bind(me)
        .fetch_all(&state.pool)
        .await
        .map_err(db)
}

async fn one(state: &AppState, board: Uuid, id: Uuid, me: Option<i64>) -> Result<Comment> {
    sqlx::query_as::<_, Comment>(concat!("SELECT ", columns!(), " WHERE c.board_id = $1 AND c.id = $3"))
        .bind(board)
        .bind(me)
        .bind(id)
        .fetch_optional(&state.pool)
        .await
        .map_err(db)?
        .ok_or_else(|| Error::not_found("comment not found"))
}

async fn anchor(state: &AppState, board: Uuid, request: &CreateRequest) -> Result<Anchor> {
    let Some(parent) = request.parent else {
        let anchor = request.anchor.clone().ok_or_else(|| Error::invalid("a comment needs an anchor"))?;
        return anchor.check().map(|()| anchor);
    };
    let found: Option<sqlx::types::Json<Anchor>> =
        sqlx::query_scalar("SELECT anchor FROM comments WHERE id = $1 AND board_id = $2 AND parent_id IS NULL")
            .bind(parent)
            .bind(board)
            .fetch_optional(&state.pool)
            .await
            .map_err(db)?;
    found.map(|json| json.0).ok_or_else(|| Error::invalid("parent is not a thread of this board"))
}

/// Adds a comment or a reply to `board`; the caller's access was checked.
pub async fn create(state: &AppState, board: Uuid, author: Author, request: CreateRequest) -> Result<Comment> {
    let body = text(&request.body)?;
    let anchor = anchor(state, board, &request).await?;
    let (user, guest, share) = match author {
        Author::User(user) => (Some(user), None, None),
        Author::Guest(name, share) => (None, Some(name), Some(share)),
    };
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO comments (board_id, parent_id, anchor, body, author_user_id, guest_name, share_id) \
         VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id",
    )
    .bind(board)
    .bind(request.parent)
    .bind(sqlx::types::Json(&anchor))
    .bind(&body)
    .bind(user)
    .bind(guest)
    .bind(share)
    .fetch_one(&state.pool)
    .await
    .map_err(db)?;
    notify(state, board).await;
    one(state, board, id, user).await
}

/// Edits a comment: a new body by its author, or resolves it (any commenter).
pub async fn update(state: &AppState, user: i64, board: Uuid, id: Uuid, patch: PatchRequest) -> Result<Comment> {
    access::check(state, user, board, Access::Commenter).await?;
    let current = one(state, board, id, Some(user)).await?;
    let body = patch.body.as_deref().map(text).transpose()?;
    if body.is_some() && !current.mine {
        return Err(Error::forbidden("only its author may edit a comment"));
    }
    sqlx::query(
        "UPDATE comments SET body = COALESCE($3, body), updated_at = now(), resolved_at = CASE \
         WHEN $4::boolean IS NULL THEN resolved_at WHEN $4 THEN COALESCE(resolved_at, now()) ELSE NULL END \
         WHERE id = $1 AND board_id = $2",
    )
    .bind(id)
    .bind(board)
    .bind(body)
    .bind(patch.resolved)
    .execute(&state.pool)
    .await
    .map_err(db)?;
    notify(state, board).await;
    one(state, board, id, Some(user)).await
}

/// Deletes a comment and its replies: its author, or an owner of the board.
pub async fn delete(state: &AppState, user: i64, board: Uuid, id: Uuid) -> Result<()> {
    let have = access::check(state, user, board, Access::Commenter).await?;
    let current = one(state, board, id, Some(user)).await?;
    if !current.mine && have < Access::Owner {
        return Err(Error::forbidden("only its author or an owner may delete a comment"));
    }
    sqlx::query("DELETE FROM comments WHERE id = $1").bind(id).execute(&state.pool).await.map_err(db)?;
    notify(state, board).await;
    Ok(())
}
