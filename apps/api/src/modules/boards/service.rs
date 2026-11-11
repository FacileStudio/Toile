use tronc::{Error, Result};
use uuid::Uuid;

use super::access::{self, Access};
use super::types::{Board, BoardRow, CreateRequest, NameRequest, columns};
use crate::extract::db;
use crate::modules::board;
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

/// The boards of one scope, most recently changed first: the live ones, or with `trash` the
/// trashed ones, which only an owner of the scope may list.
pub async fn list(state: &AppState, user_id: i64, space_id: Option<&str>, trash: bool) -> Result<Vec<Board>> {
    let have = access::scope(state, user_id, space_id).await?;
    if trash {
        access::require(have, Access::Owner)?;
    }
    let rows = sqlx::query_as::<_, BoardRow>(concat!(
        "SELECT ",
        columns!(),
        " FROM boards WHERE (deleted_at IS NOT NULL) = $3 AND ",
        "(($1::text IS NULL AND space_id IS NULL AND owner_user_id = $2) OR space_id = $1) ",
        "ORDER BY updated_at DESC, id"
    ))
    .bind(space_id)
    .bind(user_id)
    .bind(trash)
    .fetch_all(&state.pool)
    .await
    .map_err(db)?;
    Ok(rows.into_iter().map(|row| Board::from_row(row, have)).collect())
}

async fn insert(state: &AppState, user_id: i64, space_id: Option<&str>, name: &str, doc: &[u8]) -> Result<BoardRow> {
    let mut tx = state.pool.begin().await.map_err(db)?;
    let row = sqlx::query_as::<_, BoardRow>(concat!(
        "INSERT INTO boards (owner_user_id, space_id, name) VALUES ($1, $2, $3) RETURNING ",
        columns!()
    ))
    .bind(user_id)
    .bind(space_id)
    .bind(name)
    .fetch_one(&mut *tx)
    .await
    .map_err(db)?;
    sqlx::query("INSERT INTO board_state (board_id, snapshot) VALUES ($1, $2)")
        .bind(row.id)
        .bind(doc)
        .execute(&mut *tx)
        .await
        .map_err(db)?;
    tx.commit().await.map_err(db)?;
    Ok(row)
}

/// Creates an empty board (`meta.schema = 1`) in the request's scope, owned by the caller.
pub async fn create(state: &AppState, user_id: i64, request: CreateRequest) -> Result<Board> {
    let name = name(&request.name)?;
    let space_id = request.space_id.as_deref();
    let have = access::scope(state, user_id, space_id).await?;
    access::require(have, Access::Editor)?;
    let row = insert(state, user_id, space_id, &name, &board::seed()).await?;
    Ok(Board::from_row(row, have))
}

/// The `boards` row of `id`, live or trashed; no access check.
pub async fn fetch(state: &AppState, id: Uuid) -> Result<BoardRow> {
    sqlx::query_as::<_, BoardRow>(concat!("SELECT ", columns!(), " FROM boards WHERE id = $1"))
        .bind(id)
        .fetch_optional(&state.pool)
        .await
        .map_err(db)?
        .ok_or_else(|| Error::not_found("board not found"))
}

/// One board the caller may see.
pub async fn read(state: &AppState, user_id: i64, id: Uuid) -> Result<Board> {
    let have = access::check(state, user_id, id, Access::Viewer).await?;
    Ok(Board::from_row(fetch(state, id).await?, have))
}

/// Renames a board; needs `editor`.
pub async fn rename(state: &AppState, user_id: i64, id: Uuid, request: NameRequest) -> Result<Board> {
    let have = access::check(state, user_id, id, Access::Editor).await?;
    let Some(raw) = request.name else { return Ok(Board::from_row(fetch(state, id).await?, have)) };
    let row = sqlx::query_as::<_, BoardRow>(concat!(
        "UPDATE boards SET name = $2, updated_at = now() WHERE id = $1 RETURNING ",
        columns!()
    ))
    .bind(id)
    .bind(name(&raw)?)
    .fetch_one(&state.pool)
    .await
    .map_err(db)?;
    Ok(Board::from_row(row, have))
}

/// Copies a board's live content into a new board of the same scope, owned by the caller; needs
/// `editor` in that scope.
pub async fn duplicate(state: &AppState, user_id: i64, id: Uuid, request: NameRequest) -> Result<Board> {
    access::check(state, user_id, id, Access::Viewer).await?;
    let source = fetch(state, id).await?;
    let have = access::scope(state, user_id, source.space_id.as_deref()).await?;
    access::require(have, Access::Editor)?;
    let copied: String = format!("{} (copie)", source.name).chars().take(MAX_NAME).collect();
    let name = name(request.name.as_deref().unwrap_or(&copied))?;
    let doc = state.hub.read(state, id).await?;
    let row = insert(state, user_id, source.space_id.as_deref(), &name, &doc).await?;
    Ok(Board::from_row(row, have))
}
