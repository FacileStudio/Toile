//! What a share link's holder reaches without a session: the board and its live socket.

use axum::extract::ws::WebSocketUpgrade;
use axum::extract::ws::rejection::WebSocketUpgradeRejection;
use axum::extract::{Path, State};
use axum::response::Response;
use serde::{Deserialize, Serialize};
use tronc::json::Json;
use tronc::{Error, Result};

use super::service::{self, Role};
use crate::extract::Params;
use crate::modules::boards::service::fetch;
use crate::modules::boards::types::Board;
use crate::modules::rooms::{Caller, Who, accept};
use crate::state::AppState;

/// The longest guest display name, in characters.
pub const MAX_GUEST_NAME: usize = 40;

/// A guest's display name: trimmed, 1 to [`MAX_GUEST_NAME`] characters, no control characters.
pub fn guest_name(raw: &str) -> Result<String> {
    let name = raw.trim();
    let count = name.chars().count();
    if count == 0 || count > MAX_GUEST_NAME {
        return Err(Error::invalid(format!("name must be 1 to {MAX_GUEST_NAME} characters")));
    }
    if name.chars().any(char::is_control) {
        return Err(Error::invalid("name must not hold control characters"));
    }
    Ok(name.to_owned())
}

/// `GET /share/{token}`: the shared board and what the link allows.
#[derive(Debug, Serialize)]
pub struct Shared {
    pub board: Board,
    pub role: Role,
}

/// `?name=`: the guest's display name.
#[derive(Debug, Default, Deserialize)]
pub struct NameQuery {
    #[serde(default)]
    pub name: String,
}

/// `GET /share/{token}`.
pub async fn board(State(state): State<AppState>, Path(token): Path<String>) -> Result<Json<Shared>> {
    let share = service::resolve(&state, &token).await?;
    let mut board = Board::from_row(fetch(&state, share.board_id).await?, share.role.access());
    board.thumbnail_url = None;
    Ok(Json(Shared { board, role: share.role }))
}

/// `GET /share/{token}/ws?name=`: the guest's y-websocket socket. An `edit` link draws; a `view`
/// or `comment` link sends presence only.
pub async fn socket(
    State(state): State<AppState>,
    Path(token): Path<String>,
    Params(query): Params<NameQuery>,
    upgrade: std::result::Result<WebSocketUpgrade, WebSocketUpgradeRejection>,
) -> Result<Response> {
    let name = guest_name(&query.name)?;
    let share = service::resolve(&state, &token).await?;
    let who = Who { user_id: None, share: Some(share.id), name, writer: share.role == Role::Edit };
    Ok(accept(state, share.board_id, Caller { who, session_id: 0 }, upgrade))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guest_names_are_trimmed_bounded_and_printable() {
        assert_eq!(guest_name("  Ana ").unwrap(), "Ana");
        assert!(guest_name(&"é".repeat(MAX_GUEST_NAME)).is_ok());
        for bad in ["", "   ", "a\u{7}b", "line\nbreak"] {
            assert_eq!(guest_name(bad).unwrap_err().code, tronc::Code::InvalidArgument, "{bad:?}");
        }
        assert!(guest_name(&"x".repeat(MAX_GUEST_NAME + 1)).is_err());
    }
}
