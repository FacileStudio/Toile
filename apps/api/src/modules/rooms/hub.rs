use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

use super::command::{Command, Gone, Reply};
use super::room;
use crate::state::AppState;

/// How many commands wait for one room before senders block.
pub const QUEUE: usize = 1024;

type Rooms = HashMap<Uuid, (u64, mpsc::Sender<Command>)>;

/// Every open room, keyed by board. One replica only: two would split a room.
#[derive(Clone, Default)]
pub struct Hub {
    rooms: Arc<Mutex<Rooms>>,
    ids: Arc<AtomicU64>,
}

impl Hub {
    /// A fresh id, for a connection or a room generation.
    pub fn next_id(&self) -> u64 {
        self.ids.fetch_add(1, Ordering::Relaxed) + 1
    }

    pub(super) fn rooms(&self) -> MutexGuard<'_, Rooms> {
        self.rooms.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(super) fn sender(&self, state: &AppState, id: Uuid) -> mpsc::Sender<Command> {
        let mut rooms = self.rooms();
        if let Some((_, sender)) = rooms.get(&id).filter(|(_, sender)| !sender.is_closed()) {
            return sender.clone();
        }
        let (sender, receiver) = mpsc::channel(QUEUE);
        let generation = self.next_id();
        rooms.insert(id, (generation, sender.clone()));
        tokio::spawn(room::run(state.clone(), id, generation, receiver));
        sender
    }

    /// Drops the room `id` of `generation` from the map; a newer room is left alone.
    pub fn forget(&self, id: Uuid, generation: u64) {
        let mut rooms = self.rooms();
        if rooms.get(&id).is_some_and(|(current, _)| *current == generation) {
            rooms.remove(&id);
        }
    }

    /// Sends a command that answers, loading the room if needed and retrying on a room that
    /// was unloading.
    pub async fn ask<T>(
        &self,
        state: &AppState,
        id: Uuid,
        mut make: impl FnMut(Reply<T>) -> Command,
    ) -> tronc::Result<T> {
        for _ in 0..3 {
            let (reply, answer) = oneshot::channel();
            if self.sender(state, id).send(make(reply)).await.is_err() {
                continue;
            }
            match answer.await {
                Ok(Ok(result)) => return result,
                Ok(Err(Gone)) | Err(_) => continue,
            }
        }
        Err(tronc::Error::unavailable("the board room is restarting; retry"))
    }

    /// Sends a command to the room of `id` when it is open; a closed room drops it.
    pub async fn tell(&self, id: Uuid, command: Command) {
        let sender = self.rooms().get(&id).map(|(_, sender)| sender.clone());
        if let Some(sender) = sender {
            _ = sender.send(command).await;
        }
    }

    /// Sends a command to every open room.
    pub async fn tell_all(&self, make: impl Fn() -> Command) {
        let senders: Vec<_> = self.rooms().values().map(|(_, sender)| sender.clone()).collect();
        for sender in senders {
            _ = sender.send(make()).await;
        }
    }
}
