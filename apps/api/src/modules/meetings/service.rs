use serde::{Deserialize, Serialize};
use tronc::{Error, Result};
use uuid::Uuid;

use crate::extract::db;
use crate::modules::boards::service as boards;
use crate::modules::boards::types::{Board, CreateRequest};
use crate::modules::share::service::{self as share, Role};
use crate::state::AppState;

/// The longest Echo room id, in characters.
pub const MAX_ROOM: usize = 200;

/// `POST /boards/meeting`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MeetingRequest {
    /// The Echo room id.
    pub room: String,
    /// The board name on first creation; `Réunion` when absent.
    pub name: Option<String>,
    /// The space the board goes in on first creation; personal when absent.
    pub space_id: Option<String>,
}

/// The edit link participants open.
#[derive(Debug, Serialize)]
pub struct MeetingShare {
    pub id: Uuid,
    /// `<origin>/s/<token>`, the same on every call while it lives.
    pub url: String,
    pub role: Role,
}

/// The meeting board and its edit link.
#[derive(Debug, Serialize)]
pub struct Meeting {
    pub board: Board,
    pub share: MeetingShare,
    /// True when this call made the board (answered as 201, not serialized).
    #[serde(skip)]
    pub created: bool,
}

type Row = (Uuid, Uuid, String);

const LIVE: &str = "SELECT m.board_id, m.share_id, m.token FROM board_meetings m \
    JOIN boards b ON b.id = m.board_id AND b.deleted_at IS NULL \
    JOIN share_links s ON s.id = m.share_id AND s.revoked_at IS NULL \
    WHERE m.owner_user_id = $1 AND m.room = $2";

const CLAIM: &str = "INSERT INTO board_meetings (owner_user_id, room, board_id, share_id, token) \
    VALUES ($1, $2, $3, $4, $5) ON CONFLICT (owner_user_id, room) DO UPDATE \
    SET board_id = EXCLUDED.board_id, share_id = EXCLUDED.share_id, token = EXCLUDED.token, created_at = now() \
    WHERE NOT EXISTS (SELECT 1 FROM boards b JOIN share_links s ON s.board_id = b.id \
      WHERE b.id = board_meetings.board_id AND s.id = board_meetings.share_id \
      AND b.deleted_at IS NULL AND s.revoked_at IS NULL) \
    RETURNING board_id";

fn room(raw: &str) -> Result<String> {
    let room = raw.trim();
    match room.chars().count() {
        0 => Err(Error::invalid("room is empty")),
        n if n > MAX_ROOM => Err(Error::invalid(format!("room is longer than {MAX_ROOM} characters"))),
        _ if room.chars().any(char::is_control) => Err(Error::invalid("room holds control characters")),
        _ => Ok(room.to_owned()),
    }
}

fn seal(state: &AppState, token: &str) -> Result<String> {
    match &state.sealer {
        Some(sealer) => sealer.seal(token).map_err(|e| Error::internal("no randomness to seal a link", e)),
        None => Ok(token.to_owned()),
    }
}

fn open_token(state: &AppState, stored: &str) -> Result<String> {
    match &state.sealer {
        Some(sealer) => sealer.open(stored).map_err(|e| Error::internal("unreadable meeting link", e)),
        None => Ok(stored.to_owned()),
    }
}

async fn live(state: &AppState, user: i64, room: &str) -> Result<Option<Row>> {
    sqlx::query_as(LIVE).bind(user).bind(room).fetch_optional(&state.pool).await.map_err(db)
}

async fn answer(state: &AppState, user: i64, (board, share, stored): Row, created: bool) -> Result<Meeting> {
    let board = boards::read(state, user, board).await?;
    let url = format!("{}/s/{}", state.public_url, open_token(state, &stored)?);
    Ok(Meeting { board, share: MeetingShare { id: share, url, role: Role::Edit }, created })
}

async fn make(state: &AppState, user: i64, room: &str, request: MeetingRequest) -> Result<Option<Row>> {
    let name = request.name.unwrap_or_else(|| "Réunion".to_owned());
    let board = boards::create(state, user, CreateRequest { name, space_id: request.space_id }).await?;
    let link = share::create(state, user, board.id, Role::Edit).await?;
    let stored = seal(state, &link.token)?;
    let claimed: Option<Uuid> = sqlx::query_scalar(CLAIM)
        .bind(user)
        .bind(room)
        .bind(board.id)
        .bind(link.id)
        .bind(&stored)
        .fetch_optional(&state.pool)
        .await
        .map_err(db)?;
    if claimed.is_none() {
        sqlx::query("DELETE FROM boards WHERE id = $1").bind(board.id).execute(&state.pool).await.map_err(db)?;
        return Ok(None);
    }
    Ok(Some((board.id, link.id, stored)))
}

/// The meeting board of `room` for `user`: the live one, or a new board and edit link. A
/// concurrent first call that loses the race drops its board and answers the winner's.
pub async fn open(state: &AppState, user: i64, request: MeetingRequest) -> Result<Meeting> {
    let room = room(&request.room)?;
    if let Some(found) = live(state, user, &room).await? {
        return answer(state, user, found, false).await;
    }
    match make(state, user, &room, request).await? {
        Some(made) => answer(state, user, made, true).await,
        None => {
            let found = live(state, user, &room).await?.ok_or_else(|| Error::unavailable("retry"))?;
            answer(state, user, found, false).await
        }
    }
}
