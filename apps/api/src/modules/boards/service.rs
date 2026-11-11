use tronc::{Error, Result};
use uuid::Uuid;

use super::access::{self, Access};
use super::types::{Board, BoardRow, CreateRequest, columns};
use crate::extract::db;
use crate::state::AppState;

/// The longest board name, in characters.
pub const MAX_NAME: usize = 200;

/// The name a board gets when the request leaves it blank.
pub const DEFAULT_NAME: &str = "Sans titre";

/// A trimmed name of at most [`MAX_NAME`] characters; blank becomes [`DEFAULT_NAME`].
pub fn name(raw: &str) -> Result<String> {
    let name = raw.trim();
    match name.chars().count() {
        0 => Ok(DEFAULT_NAME.to_owned()),
        n if n > MAX_NAME => Err(Error::invalid(format!("name is longer than {MAX_NAME} characters"))),
        _ => Ok(name.to_owned()),
    }
}

/// The live boards of one scope, most recently changed first.
pub async fn list(state: &AppState, user_id: i64, space_id: Option<&str>) -> Result<Vec<Board>> {
    let have = access::scope(state, user_id, space_id).await?;
    let rows = sqlx::query_as::<_, BoardRow>(concat!(
        "SELECT ",
        columns!(),
        " FROM boards WHERE deleted_at IS NULL AND ",
        "(($1::text IS NULL AND space_id IS NULL AND owner_user_id = $2) OR space_id = $1) ",
        "ORDER BY updated_at DESC, id"
    ))
    .bind(space_id)
    .bind(user_id)
    .fetch_all(&state.pool)
    .await
    .map_err(db)?;
    Ok(rows.into_iter().map(|row| Board::from_row(row, have)).collect())
}

/// Creates a board in the request's scope, owned by the caller.
pub async fn create(state: &AppState, user_id: i64, request: CreateRequest) -> Result<Board> {
    let name = name(&request.name)?;
    let have = access::scope(state, user_id, request.space_id.as_deref()).await?;
    access::require(have, Access::Editor)?;
    let row = sqlx::query_as::<_, BoardRow>(concat!(
        "INSERT INTO boards (owner_user_id, space_id, name) VALUES ($1, $2, $3) RETURNING ",
        columns!()
    ))
    .bind(user_id)
    .bind(request.space_id.as_deref())
    .bind(&name)
    .fetch_one(&state.pool)
    .await
    .map_err(db)?;
    Ok(Board::from_row(row, have))
}

/// One board the caller may see.
pub async fn read(state: &AppState, user_id: i64, id: Uuid) -> Result<Board> {
    let have = access::check(state, user_id, id, Access::Viewer).await?;
    let row = sqlx::query_as::<_, BoardRow>(concat!("SELECT ", columns!(), " FROM boards WHERE id = $1"))
        .bind(id)
        .fetch_optional(&state.pool)
        .await
        .map_err(db)?
        .ok_or_else(|| Error::not_found("board not found"))?;
    Ok(Board::from_row(row, have))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_trimmed_defaulted_and_capped() {
        assert_eq!(name("  Atelier  ").unwrap(), "Atelier");
        assert_eq!(name("   ").unwrap(), DEFAULT_NAME);
        assert!(name(&"é".repeat(MAX_NAME)).is_ok());
        assert_eq!(name(&"é".repeat(MAX_NAME + 1)).unwrap_err().code, tronc::Code::InvalidArgument);
    }
}
