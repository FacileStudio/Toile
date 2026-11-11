use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tronc::{Error, Result};
use uuid::Uuid;

use crate::extract::db;
use crate::modules::boards::access::{self, Access};
use crate::modules::rooms::Command;
use crate::state::AppState;

/// What a link lets its holder do.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "text", rename_all = "lowercase")]
pub enum Role {
    /// Look; presence only.
    View,
    /// Look and comment.
    Comment,
    /// Draw and comment.
    Edit,
}

impl Role {
    /// The board access the link gives.
    pub fn access(self) -> Access {
        match self {
            Role::View => Access::Viewer,
            Role::Comment => Access::Commenter,
            Role::Edit => Access::Editor,
        }
    }
}

/// A new link, as `POST /boards/{id}/shares` answers it: the token is shown this once.
#[derive(Debug, Serialize)]
pub struct Created {
    pub id: Uuid,
    pub token: String,
    pub url: String,
    pub role: Role,
}

/// A live link as the owner's list shows it, without its token (only its hash is stored).
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Link {
    pub id: Uuid,
    pub role: Role,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

/// A resolved token.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Share {
    pub id: Uuid,
    pub board_id: Uuid,
    pub role: Role,
}

/// A new link to `board` with `role`; needs `owner`.
pub async fn create(state: &AppState, user_id: i64, board: Uuid, role: Role) -> Result<Created> {
    access::check(state, user_id, board, Access::Owner).await?;
    let token = porte::new_token().map_err(|error| Error::internal("no randomness for a share token", error))?;
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO share_links (board_id, token_hash, role, created_by) VALUES ($1, $2, $3, $4) RETURNING id",
    )
    .bind(board)
    .bind(porte::hash_token(&token))
    .bind(role)
    .bind(user_id)
    .fetch_one(&state.pool)
    .await
    .map_err(db)?;
    Ok(Created { id, url: format!("/s/{token}"), token, role })
}

/// The live links of `board`, newest first; needs `owner`.
pub async fn list(state: &AppState, user_id: i64, board: Uuid) -> Result<Vec<Link>> {
    access::check(state, user_id, board, Access::Owner).await?;
    sqlx::query_as::<_, Link>(
        "SELECT id, role, created_at FROM share_links WHERE board_id = $1 AND revoked_at IS NULL \
         ORDER BY created_at DESC, id",
    )
    .bind(board)
    .fetch_all(&state.pool)
    .await
    .map_err(db)
}

/// Revokes a link of `board` and closes the sockets opened with it; needs `owner`.
pub async fn revoke(state: &AppState, user_id: i64, board: Uuid, id: Uuid) -> Result<()> {
    access::check(state, user_id, board, Access::Owner).await?;
    let done =
        sqlx::query("UPDATE share_links SET revoked_at = now() WHERE id = $1 AND board_id = $2 AND revoked_at IS NULL")
            .bind(id)
            .bind(board)
            .execute(&state.pool)
            .await
            .map_err(db)?;
    if done.rows_affected() == 0 {
        return Err(Error::not_found("share link not found"));
    }
    state.hub.tell(board, Command::Revoke(id)).await;
    Ok(())
}

/// The live link `token` names, on a live board; anything else is `404`.
pub async fn resolve(state: &AppState, token: &str) -> Result<Share> {
    sqlx::query_as::<_, Share>(
        "SELECT s.id, s.board_id, s.role FROM share_links s JOIN boards b ON b.id = s.board_id \
         WHERE s.token_hash = $1 AND s.revoked_at IS NULL AND b.deleted_at IS NULL",
    )
    .bind(porte::hash_token(token))
    .fetch_optional(&state.pool)
    .await
    .map_err(db)?
    .ok_or_else(|| Error::not_found("share link not found"))
}
