use tronc::{Error, Result};
use uuid::Uuid;

use crate::extract::db;
use crate::state::AppState;

/// What a caller may do with one board. Every board query goes through [`access`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Access {
    /// The board does not exist for this caller: always `404`.
    None,
    /// Look only; presence is relayed.
    Viewer,
    /// Look and comment.
    Commenter,
    /// Draw, comment and rename.
    Editor,
    /// Edit, trash, restore, delete and share.
    Owner,
}

impl Access {
    /// The name the API answers in `Board.role`.
    pub fn name(self) -> &'static str {
        match self {
            Access::None => "none",
            Access::Viewer => "viewer",
            Access::Commenter => "commenter",
            Access::Editor => "editor",
            Access::Owner => "owner",
        }
    }
}

/// The spaces of a user, as `(space_id, role)` rows: `$1` is the porte user id, `$2` the issuer.
macro_rules! member_spaces {
    () => {
        "SELECT m.space_id, m.role FROM space_members m \
         JOIN spaces live ON live.id = m.space_id AND live.deleted_at IS NULL \
         JOIN porte_identities i ON i.subject = m.subject AND i.provider = $2 WHERE i.user_id = $1"
    };
}
pub(crate) use member_spaces;

/// The access a membership role gives on the space's boards.
pub fn from_role(role: &str) -> Access {
    match role {
        "admin" | "owner" => Access::Owner,
        "member" => Access::Editor,
        _ => Access::None,
    }
}

/// The caller's access to `board`, live (`trashed` false) or in the trash (`trashed` true). A
/// personal board belongs to its owner alone; a space board follows the caller's membership. An
/// instance admin gets no bypass.
pub async fn lookup(state: &AppState, user_id: i64, board: Uuid, trashed: bool) -> Result<Access> {
    let row: Option<(Option<String>, Option<i64>, Option<String>)> = sqlx::query_as(concat!(
        "SELECT b.space_id, b.owner_user_id, s.role FROM boards b LEFT JOIN (",
        member_spaces!(),
        ") s ON s.space_id = b.space_id WHERE b.id = $3 AND (b.deleted_at IS NOT NULL) = $4"
    ))
    .bind(user_id)
    .bind(&*state.issuer)
    .bind(board)
    .bind(trashed)
    .fetch_optional(&state.pool)
    .await
    .map_err(db)?;
    Ok(match row {
        None => Access::None,
        Some((None, owner, _)) if owner == Some(user_id) => Access::Owner,
        Some((None, _, _)) => Access::None,
        Some((Some(_), _, role)) => role.as_deref().map_or(Access::None, from_role),
    })
}

/// The caller's access to the live `board`: every board query goes through it. A trashed board
/// is `None`.
pub async fn access(state: &AppState, user_id: i64, board: Uuid) -> Result<Access> {
    lookup(state, user_id, board, false).await
}

/// Refuses unless `have` reaches `need`: `404` when the caller cannot see the board, `403`
/// when it can but may not do this.
pub fn require(have: Access, need: Access) -> Result<()> {
    match have {
        Access::None => Err(Error::not_found("board not found")),
        have if have < need => Err(Error::forbidden("you may not do this to the board")),
        _ => Ok(()),
    }
}

/// `access` then `require`, in one call.
pub async fn check(state: &AppState, user_id: i64, board: Uuid, need: Access) -> Result<Access> {
    let have = access(state, user_id, board).await?;
    require(have, need)?;
    Ok(have)
}

/// Resolves the `?space_id=` scope of a list or a create: `None` is the personal scope, a space
/// the caller is not a member of is `404`. Answers the caller's access in that scope.
pub async fn scope(state: &AppState, user_id: i64, space_id: Option<&str>) -> Result<Access> {
    let Some(space_id) = space_id else { return Ok(Access::Owner) };
    let role: Option<String> =
        sqlx::query_scalar(concat!("SELECT role FROM (", member_spaces!(), ") s WHERE space_id = $3"))
            .bind(user_id)
            .bind(&*state.issuer)
            .bind(space_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(db)?;
    match role.as_deref().map_or(Access::None, from_role) {
        Access::None => Err(Error::not_found("space not found")),
        access => Ok(access),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roles_map_to_access_and_none_is_not_found_while_too_little_is_forbidden() {
        assert_eq!(from_role("member"), Access::Editor);
        assert_eq!(from_role("admin"), Access::Owner);
        assert_eq!(from_role("owner"), Access::Owner);
        assert_eq!(from_role("guest"), Access::None);
        assert_eq!(require(Access::None, Access::Viewer).unwrap_err().code, tronc::Code::NotFound);
        assert_eq!(require(Access::Commenter, Access::Editor).unwrap_err().code, tronc::Code::PermissionDenied);
        assert!(require(Access::Owner, Access::Editor).is_ok());
    }
}
