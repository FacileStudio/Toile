//! Whether an open socket may stay: its session row, or its share link, still stands.

use std::time::Duration;

use salon::Caller;
use tokio::time::Instant;

use crate::state::AppState;

/// How long a socket opened with a bearer JWT (no session row) lives before it must reconnect.
pub const MAX_BEARER_AGE: Duration = Duration::from_secs(15 * 60);

async fn live(state: &AppState, caller: &Caller) -> Result<bool, sqlx::Error> {
    match (caller.who.user_id, caller.who.share) {
        (_, Some(share)) => {
            sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM share_links WHERE id = $1 AND revoked_at IS NULL)")
                .bind(share)
                .fetch_one(&state.pool)
                .await
        }
        (Some(user), None) => {
            sqlx::query_scalar(
                "SELECT EXISTS (SELECT 1 FROM porte_sessions WHERE id = $1 AND user_id = $2 \
             AND (expires_at IS NULL OR expires_at > now()))",
            )
            .bind(caller.session_id)
            .bind(user)
            .fetch_one(&state.pool)
            .await
        }
        (None, None) => Ok(false),
    }
}

/// `None` keeps the socket; a code and reason close it: the bearer token aged out, or the
/// session or share link ended.
pub async fn alive(state: &AppState, caller: &Caller, opened: Instant) -> Option<(u16, String)> {
    if caller.who.share.is_none() && caller.session_id == 0 {
        return (opened.elapsed() >= MAX_BEARER_AGE).then(|| (1001, "token age, reconnect".to_owned()));
    }
    matches!(live(state, caller).await, Ok(false)).then(|| (1008, "access revoked".to_owned()))
}
