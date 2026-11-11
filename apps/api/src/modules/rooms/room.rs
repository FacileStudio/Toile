use std::collections::{HashMap, HashSet};
use std::time::Duration;

use tokio::sync::mpsc;
use tokio::time::Instant;
use uuid::Uuid;
use yrs::ClientID;
use yrs::sync::Awareness;

use super::command::{Command, Out, Who};
use super::core::Core;
use super::drain::{answer_gone, refuse_all};
use super::load::load;
use crate::state::AppState;

/// How long accepted updates wait in memory before they are appended to `board_updates`.
pub const FLUSH: Duration = Duration::from_millis(250);
/// A snapshot is written once this many updates are pending.
pub const SNAPSHOT_EVERY: usize = 200;
/// A snapshot is written once the oldest pending update is this old.
pub const SNAPSHOT_AFTER: Duration = Duration::from_secs(60);
/// A snapshot is written once the pending updates weigh this much.
pub const SNAPSHOT_BYTES: usize = 4 << 20;
/// How long an empty room stays loaded.
pub const IDLE: Duration = Duration::from_secs(60);

pub(super) struct Conn {
    pub(super) outbox: mpsc::Sender<Out>,
    pub(super) who: Who,
    pub(super) clients: HashSet<ClientID>,
}

pub(super) struct Room {
    pub(super) state: AppState,
    pub(super) id: Uuid,
    pub(super) core: Core,
    pub(super) awareness: Awareness,
    pub(super) conns: HashMap<u64, Conn>,
    pub(super) flushed: usize,
    pub(super) flush_at: Option<Instant>,
    pub(super) since: Option<Instant>,
    pub(super) empty_since: Option<Instant>,
    pub(super) ended: bool,
}

async fn open(state: &AppState, id: Uuid) -> tronc::Result<Core> {
    let (base, log) = match load(&state.pool, id).await {
        Ok(Some(stored)) => stored,
        Ok(None) => return Err(tronc::Error::not_found("board not found")),
        Err(error) => return Err(tronc::Error::internal("failed to load a room", error)),
    };
    Core::new(base, log).map_err(|error| tronc::Error::internal("failed to read a stored board", error))
}

/// The room task: loads the board, then serves commands until it has been empty for [`IDLE`]
/// or is shut down.
pub async fn run(state: AppState, id: Uuid, generation: u64, mut commands: mpsc::Receiver<Command>) {
    let mut room = match open(&state, id).await {
        Ok(core) => Room::new(state, id, core),
        Err(error) => return refuse_all(&state, id, generation, commands, error).await,
    };
    loop {
        let deadline = room.deadline();
        tokio::select! {
            command = commands.recv() => match command {
                Some(command) => if !room.handle(command).await { break },
                None => break,
            },
            () = sleep_until(deadline) => if room.tick().await { break },
        }
    }
    room.state.hub.forget(id, generation);
    commands.close();
    while let Ok(command) = commands.try_recv() {
        answer_gone(command);
    }
}

async fn sleep_until(deadline: Option<Instant>) {
    match deadline {
        Some(deadline) => tokio::time::sleep_until(deadline).await,
        None => std::future::pending().await,
    }
}

impl Room {
    fn new(state: AppState, id: Uuid, core: Core) -> Self {
        let now = Instant::now();
        let (flushed, since) = (core.log.len(), (!core.log.is_empty()).then_some(now));
        let awareness = Awareness::new(yrs::Doc::new());
        let conns = HashMap::new();
        let (flush_at, empty_since, ended) = (None, Some(now), false);
        Room { state, id, core, awareness, conns, flushed, flush_at, since, empty_since, ended }
    }

    fn deadline(&self) -> Option<Instant> {
        if self.ended {
            return Some(Instant::now());
        }
        let snapshot = self.since.map(|at| at + SNAPSHOT_AFTER);
        let idle = self.empty_since.filter(|_| self.since.is_none()).map(|at| at + IDLE);
        [self.flush_at, snapshot, idle].into_iter().flatten().min()
    }

    async fn tick(&mut self) -> bool {
        let now = Instant::now();
        if self.snapshot_due(now) {
            self.snapshot(true).await;
        } else if self.flush_at.is_some_and(|at| at <= now) {
            self.flush().await;
        }
        self.ended || self.since.is_none() && self.empty_since.is_some_and(|at| at + IDLE <= now)
    }

    /// Queues `out` for `conn`; a member whose outbox is full is dropped with 1013.
    pub(super) fn send(&mut self, conn: u64, out: Out) {
        let full = self.conns.get(&conn).is_some_and(|c| c.outbox.try_send(out).is_err());
        if full && let Some(conn) = self.conns.remove(&conn) {
            _ = conn.outbox.try_send(Out::Close(1013, "too slow".to_owned()));
        }
    }

    /// Records an accepted update: it is appended to the store after [`FLUSH`].
    pub(super) fn accepted(&mut self) {
        let now = Instant::now();
        self.flush_at.get_or_insert(now + FLUSH);
        self.since.get_or_insert(now);
    }
}
