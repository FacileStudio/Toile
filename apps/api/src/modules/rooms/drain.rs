use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

use super::command::{Command, Gone, Out};
use super::hub::Hub;
use super::room::Room;
use crate::state::AppState;

/// Answers a command that reached a room after it stopped: the hub retries on a new room.
pub fn answer_gone(command: Command) {
    match command {
        Command::Join(_, reply) => _ = reply.send(Err(Gone)),
        Command::Read(reply) => _ = reply.send(Err(Gone)),
        Command::Write(_, reply) => _ = reply.send(Err(Gone)),
        Command::Shutdown(_, _, done) => _ = done.send(()),
        Command::Message(..) | Command::Leave(_) | Command::Broadcast(_) | Command::Evict(_) | Command::Revoke(_) => {}
    }
}

/// Answers every command with `error`: the board could not be loaded.
pub async fn refuse_all(
    state: &AppState,
    id: Uuid,
    generation: u64,
    mut commands: mpsc::Receiver<Command>,
    error: tronc::Error,
) {
    state.hub.forget(id, generation);
    commands.close();
    while let Some(command) = commands.recv().await {
        let failure = tronc::Error::new(error.code, error.message.clone());
        match command {
            Command::Join(_, reply) => _ = reply.send(Ok(Err(failure))),
            Command::Read(reply) => _ = reply.send(Ok(Err(failure))),
            Command::Write(_, reply) => _ = reply.send(Ok(Err(failure))),
            other => answer_gone(other),
        }
    }
}

async fn stop_all(senders: Vec<mpsc::Sender<Command>>, code: u16, reason: &str) {
    for sender in senders {
        let (done, wait) = oneshot::channel();
        if sender.send(Command::Shutdown(code, reason.to_owned(), done)).await.is_ok() {
            _ = wait.await;
        }
    }
}

impl Hub {
    /// Persists and unloads the rooms of `ids`, closing their members with 1008: the boards
    /// were trashed or deleted.
    pub async fn close(&self, ids: &[Uuid]) {
        let senders: Vec<_> = {
            let rooms = self.rooms();
            ids.iter().filter_map(|id| rooms.get(id).map(|(_, sender)| sender.clone())).collect()
        };
        stop_all(senders, 1008, "board deleted").await;
    }

    /// Persists and unloads every room, closing members with 1013, for a graceful stop.
    pub async fn shutdown(&self) {
        let senders: Vec<_> = self.rooms().values().map(|(_, sender)| sender.clone()).collect();
        stop_all(senders, 1013, "server restarting").await;
    }
}

impl Room {
    /// Snapshots, then closes every member with `code` and unloads.
    pub(super) async fn stop(&mut self, code: u16, reason: &str) {
        self.snapshot(false).await;
        self.end(code, reason);
    }

    pub(super) fn vanish(&mut self) {
        tracing::info!(board = %self.id, "rooms: the board was deleted, unloading");
        self.end(1008, "board deleted");
    }

    /// Unloads without saving: every member is closed with `code` and the room stops at once.
    pub(super) fn end(&mut self, code: u16, reason: &str) {
        (self.flush_at, self.since, self.ended) = (None, None, true);
        let conns: Vec<u64> = self.conns.keys().copied().collect();
        for conn in conns {
            self.drop_conn(conn, Some(Out::Close(code, reason.to_owned())));
        }
    }
}
