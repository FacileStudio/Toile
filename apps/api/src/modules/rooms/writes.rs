//! Server-side reads and writes of a board, through its room so open clients see them live.

use tokio::sync::oneshot;
use uuid::Uuid;
use yrs::sync::{Message, SyncMessage};

use super::command::{Command, Edit, Gone};
use super::core::Refusal;
use super::hub::Hub;
use super::room::Room;
use super::sync::encode;
use crate::state::AppState;

fn restarting() -> tronc::Error {
    tronc::Error::unavailable("the board room is restarting; retry")
}

impl Hub {
    /// The live state of board `id`, as one Yjs v1 update: what open clients see.
    pub async fn read(&self, state: &AppState, id: Uuid) -> tronc::Result<Vec<u8>> {
        self.ask(state, id, Command::Read).await
    }

    /// Runs `edit` inside the room of board `id`: validated like a client update, broadcast to
    /// open clients and appended to the store before the answer (a failed append is retried).
    /// An invalid edit is undone and refused (400).
    pub async fn write(&self, state: &AppState, id: Uuid, edit: Edit) -> tronc::Result<()> {
        let (reply, answer) = oneshot::channel();
        _ = self.sender(state, id).send(Command::Write(edit, reply)).await;
        match answer.await {
            Ok(Ok(result)) => result,
            Ok(Err(Gone)) | Err(_) => Err(restarting()),
        }
    }
}

impl Room {
    pub(super) async fn write(&mut self, edit: Edit) -> tronc::Result<()> {
        let update = self.core.edit(edit).map_err(|Refusal(_, reason)| tronc::Error::invalid(reason))?;
        self.broadcast(None, &encode(Message::Sync(SyncMessage::Update(update))));
        self.accepted();
        self.flush().await;
        match self.ended {
            true => Err(tronc::Error::not_found("board not found")),
            false => Ok(()),
        }
    }
}
