//! Toile's handle on its salon rooms, with the calls the rest of the API makes.

use salon::{Change, Due};
use uuid::Uuid;

use crate::state::AppState;

/// A server-side change to a board: runs in one Yjs transaction over `shapes` and is validated
/// like any client update.
pub type Edit = Box<dyn FnOnce(&mut yrs::TransactionMut, &yrs::MapRef) -> Result<(), String> + Send>;

/// Every open board room. One replica only: two would split a room.
#[derive(Clone, Default)]
pub struct Hub(pub(super) salon::Hub<AppState>);

impl Hub {
    /// The live state of board `id`, as one Yjs v1 update: what open clients see.
    pub async fn read(&self, state: &AppState, id: Uuid) -> tronc::Result<Vec<u8>> {
        self.0.read(state, id, |core| core.state()).await
    }

    /// Runs `edit` inside the room of board `id`: validated like a client update, broadcast to
    /// open clients and appended to the store before the answer (a failed append is retried).
    /// An invalid edit is undone and refused (400).
    pub async fn write(&self, state: &AppState, id: Uuid, edit: Edit) -> tronc::Result<()> {
        let edit = move |core: &mut super::core::Core| {
            core.edit(edit).map(Some).map_err(|salon::Refusal(_, reason)| tronc::Error::invalid(reason))
        };
        let change = Change { due: Due::Flush, author: None, edit, answer: |_: &_, _| () };
        self.0.write(state, id, change).await
    }

    /// Persists and unloads the rooms of `ids`, closing their members with 1008: the boards
    /// were trashed or deleted.
    pub async fn close(&self, ids: &[Uuid]) {
        self.0.close(ids, 1008, "board deleted").await;
    }

    /// Persists and unloads every room, closing members with 1013, for a graceful stop.
    pub async fn shutdown(&self) {
        self.0.shutdown(1013, "server restarting").await;
    }

    /// Sends `frame` to every member of board `id`, when its room is open.
    pub async fn broadcast(&self, id: Uuid, frame: Vec<u8>) {
        self.0.broadcast(id, frame).await;
    }

    /// Closes, in every open room, the connections of `users` that lost access.
    pub async fn evict(&self, users: &[i64]) {
        self.0.evict(users).await;
    }

    /// Closes every connection of board `id` that came through share link `share`.
    pub async fn revoke(&self, id: Uuid, share: Uuid) {
        self.0.revoke(id, share).await;
    }
}
