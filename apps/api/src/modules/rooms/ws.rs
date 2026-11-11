use std::time::Duration;

use axum::extract::ws::{CloseFrame, Message, WebSocket};
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio::time::Instant;
use uuid::Uuid;

use super::command::{Command, Member, Out, Who};
use crate::state::AppState;

/// The largest frame a member may send.
pub const MAX_FRAME: usize = 2 << 20;
/// How many frames wait for a slow member before it is closed with 1013.
pub const OUTBOX: usize = 512;
/// How often a socket re-checks its session or share link in production.
pub const SOCKET_RECHECK: Duration = Duration::from_secs(30);
/// How long a socket opened with a bearer JWT (no session row) lives before it must reconnect.
pub const MAX_BEARER_AGE: Duration = Duration::from_secs(15 * 60);
/// The longest close reason a WebSocket close frame carries, in bytes (RFC 6455 section 5.5).
pub const MAX_REASON: usize = 123;

/// A close frame whose reason is cut to [`MAX_REASON`] bytes on a character boundary: a longer
/// one fails to send and the client sees 1006 instead of the code.
pub fn close(code: u16, mut reason: String) -> Message {
    let cut = (0..=MAX_REASON.min(reason.len())).rev().find(|&at| reason.is_char_boundary(at)).unwrap_or(0);
    reason.truncate(cut);
    Message::Close(Some(CloseFrame { code, reason: reason.into() }))
}

/// Who a socket speaks for, as authorized at the upgrade.
#[derive(Debug, Clone)]
pub struct Caller {
    /// The member it joins as.
    pub who: Who,
    /// The session row behind a member's cookie; zero on the bearer-JWT path and for guests.
    pub session_id: i64,
}

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

async fn alive(state: &AppState, caller: &Caller, opened: Instant) -> Option<(u16, &'static str)> {
    if caller.who.share.is_none() && caller.session_id == 0 {
        return (opened.elapsed() >= MAX_BEARER_AGE).then_some((1001, "token age, reconnect"));
    }
    matches!(live(state, caller).await, Ok(false)).then_some((1008, "access revoked"))
}

/// Pumps one socket: joins the room, forwards frames both ways, re-checks the session or link
/// every [`AppState::socket_recheck`], leaves on close.
pub async fn serve(socket: WebSocket, state: AppState, board: Uuid, caller: Caller) {
    let (mut sink, stream) = socket.split();
    let (outbox, inbox) = mpsc::channel(OUTBOX);
    let conn = state.hub.next_id();
    let member = Member { conn, who: caller.who.clone(), outbox };
    if let Err(error) = state.hub.ask(&state, board, |reply| Command::Join(member.clone(), reply)).await {
        _ = sink.send(close(1011, error.message)).await;
        return;
    }
    pump(&state, board, conn, &caller, (sink, stream, inbox)).await;
    state.hub.tell(board, Command::Leave(conn)).await;
}

type Halves = (SplitSink<WebSocket, Message>, SplitStream<WebSocket>, mpsc::Receiver<Out>);

async fn pump(state: &AppState, board: Uuid, conn: u64, caller: &Caller, (mut sink, mut stream, mut inbox): Halves) {
    let opened = Instant::now();
    let mut recheck = tokio::time::interval_at(opened + state.socket_recheck, state.socket_recheck);
    loop {
        tokio::select! {
            incoming = stream.next() => match incoming {
                Some(Ok(Message::Binary(frame))) if frame.len() > MAX_FRAME => {
                    _ = sink.send(close(1009, "frame over 2 MiB".to_owned())).await;
                    break;
                }
                Some(Ok(Message::Binary(frame))) => state.hub.tell(board, Command::Message(conn, frame.to_vec())).await,
                Some(Ok(Message::Close(_))) | None | Some(Err(_)) => break,
                Some(Ok(_)) => {}
            },
            out = inbox.recv() => match out {
                Some(Out::Binary(frame)) => if sink.send(Message::Binary(frame.into())).await.is_err() { break },
                Some(Out::Close(code, reason)) => { _ = sink.send(close(code, reason)).await; break }
                None => break,
            },
            _ = recheck.tick() => if let Some((code, reason)) = alive(state, caller, opened).await {
                _ = sink.send(close(code, reason.to_owned())).await;
                break;
            },
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_long_close_reason_is_cut_on_a_character_boundary() {
        let axum::extract::ws::Message::Close(Some(frame)) = super::close(1008, "é".repeat(100)) else { panic!() };
        assert_eq!((frame.code, frame.reason.len()), (1008, 122));
    }
}
