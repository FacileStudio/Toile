//! Board shapes as plain values: read from a board's `shapes`, rendered as text for agents,
//! planned with an auto-layout, and written back in one room transaction (MCP and AI).

pub mod change;
pub mod input;
pub mod layout;
pub mod plan;
pub mod read;
pub mod text;
pub mod write;

#[cfg(test)]
mod tests;

use uuid::Uuid;

use crate::modules::board::{self, SHAPES};
use crate::state::AppState;

/// The live shapes of board `id`, in paint order, as open clients see them.
pub async fn load(state: &AppState, id: Uuid) -> tronc::Result<Vec<read::Shape>> {
    let bytes = state.hub.read(state, id).await?;
    let doc = board::from_state(&bytes, []).map_err(|error| tronc::Error::internal("unreadable board", error))?;
    let shapes = doc.get_or_insert_map(SHAPES);
    let txn = yrs::Transact::transact(&doc);
    Ok(read::all(&txn, &shapes))
}

/// A new shape id: 16 url-safe characters, like the client's.
pub fn fresh_id() -> String {
    Uuid::new_v4().simple().to_string()[..16].to_owned()
}
