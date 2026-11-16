//! Toile's side of the rooms: how a board loads, saves, and who may stay in its room.

use std::time::Duration;

use salon::{Caller, Due, Fault, Kind, Policy, Saved};
use tokio::time::Instant;
use tronc::Code;
use uuid::Uuid;

use super::core::Core;
use super::live::alive;
use super::load::load;
use super::persist::{append, store};
use crate::modules::boards::access::{self, Access};
use crate::state::AppState;

/// How long accepted updates wait in memory before they are appended to `board_updates`.
const FLUSH: Duration = Duration::from_millis(250);

fn code(kind: Kind) -> Code {
    match kind {
        Kind::NotFound => Code::NotFound,
        Kind::Invalid => Code::InvalidArgument,
        Kind::Failed => Code::FailedPrecondition,
        Kind::Unavailable => Code::Unavailable,
        Kind::Internal => Code::Internal,
    }
}

fn stored(board: Uuid, outcome: Result<bool, sqlx::Error>, what: &str) -> Option<bool> {
    match outcome {
        Ok(kept) => Some(kept),
        Err(error) => {
            tracing::error!(board = %board, error = %error, "rooms: {what} failed, retrying");
            None
        }
    }
}

impl salon::Host for AppState {
    type Space = Core;
    type Error = tronc::Error;

    fn rooms(&self) -> (&salon::Hub<Self>, Policy) {
        let policy = Policy {
            flush: FLUSH,
            debounce: false,
            retry: FLUSH * 4,
            snapshot_every: 200,
            snapshot_bytes: 4 << 20,
            snapshot_after: Some(Duration::from_secs(60)),
            idle: Duration::from_secs(60),
            recheck: self.socket_recheck,
            blocking: false,
            noun: "board",
        };
        (&self.hub.0, policy)
    }

    fn fault(fault: Fault) -> tronc::Error {
        tronc::Error::new(code(fault.kind), fault.message)
    }

    async fn open(&self, key: Uuid) -> Result<Core, Fault> {
        let (base, log) = match load(&self.pool, key).await {
            Ok(Some(stored)) => stored,
            Ok(None) => return Err(Fault::new(Kind::NotFound, "board not found")),
            Err(error) => {
                tracing::error!(board = %key, error = %error, "rooms: failed to load a room");
                return Err(Fault::new(Kind::Internal, "failed to load a room"));
            }
        };
        Core::new(base, log).map_err(|_| Fault::new(Kind::Internal, "failed to read a stored board"))
    }

    async fn save(&self, key: Uuid, core: &mut Core, due: Due, _author: Option<i64>) -> Saved {
        if due == Due::Flush {
            let outcome = append(&self.pool, key, &core.log[core.flushed..]).await;
            return match stored(key, outcome, "append") {
                Some(true) => {
                    core.flushed = core.log.len();
                    Saved::Flushed
                }
                Some(false) => Saved::Gone,
                None => Saved::Retry,
            };
        }
        let state = core.state();
        match stored(key, store(&self.pool, key, &state).await, "snapshot") {
            Some(true) => {
                core.compacted(state);
                Saved::Stored
            }
            Some(false) => Saved::Gone,
            None => Saved::Retry,
        }
    }

    async fn alive(&self, caller: &Caller, opened: Instant) -> Option<(u16, String)> {
        alive(self, caller, opened).await
    }

    async fn lost(&self, key: Uuid, user: i64) -> bool {
        matches!(access::access(self, user, key).await, Ok(Access::None))
    }
}
