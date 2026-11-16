//! A room's document: the Y.Doc, its validator, and the stored state it can always fall back to.

use std::sync::{Arc, Mutex};

use salon::{Refusal, Space};
use yrs::{Doc, Transact};

use super::hub::Edit;
use crate::modules::board::schema::{self, Touched};
use crate::modules::board::{self, DecodeError, SHAPES};

/// The document of one room. `base` and `log` are what is stored: the last snapshot and every
/// update accepted since, so a refused update reverts to exactly the state before it.
pub struct Core {
    /// The live document.
    pub doc: Doc,
    pub(super) touched: Arc<Mutex<Touched>>,
    /// The last snapshot, a full v1 update.
    pub base: Vec<u8>,
    /// Every update accepted since `base`, in order.
    pub log: Vec<Vec<u8>>,
    /// The bytes of `log`.
    pub bytes: usize,
    /// How many updates of `log` are already in `board_updates`.
    pub flushed: usize,
}

impl Default for Core {
    fn default() -> Self {
        let doc = board::new_doc();
        let touched = schema::watch(&doc);
        Core { doc, touched, base: Vec::new(), log: Vec::new(), bytes: 0, flushed: 0 }
    }
}

impl Core {
    /// The document of `base` then `log`, every update of `log` already stored.
    pub fn new(base: Vec<u8>, log: Vec<Vec<u8>>) -> Result<Core, DecodeError> {
        let doc = board::from_state(&base, log.iter().map(Vec::as_slice))?;
        let touched = schema::watch(&doc);
        let (bytes, flushed) = (log.iter().map(Vec::len).sum(), log.len());
        Ok(Core { doc, touched, base, log, bytes, flushed })
    }

    pub(super) fn take(&self) -> Touched {
        std::mem::take(&mut *self.touched.lock().unwrap_or_else(|poisoned| poisoned.into_inner()))
    }

    pub(super) fn settle(&mut self, outcome: Result<Vec<u8>, Refusal>) -> Result<Vec<u8>, Refusal> {
        let touched = self.take();
        let checked = outcome.and_then(|update| {
            schema::check(&self.doc, &touched).map(|()| update).map_err(|reason| Refusal(1008, reason))
        });
        match checked {
            Ok(update) => {
                self.bytes += update.len();
                self.log.push(update.clone());
                Ok(update)
            }
            Err(refusal) => self.recover().and(Err(refusal)),
        }
    }

    /// Runs a server-side `edit` as one transaction; answers its update, or undoes it.
    pub fn edit(&mut self, edit: Edit) -> Result<Vec<u8>, Refusal> {
        let shapes = self.doc.get_or_insert_map(SHAPES);
        let outcome = {
            let mut txn = self.doc.transact_mut();
            edit(&mut txn, &shapes).map(|()| txn.encode_update_v1()).map_err(|reason| Refusal(1008, reason))
        };
        self.settle(outcome)
    }

    /// The whole state, as one v1 update.
    pub fn state(&self) -> Vec<u8> {
        board::encode(&self.doc)
    }

    /// Makes the current state the new `base` and empties `log`, after a snapshot.
    pub fn compacted(&mut self, state: Vec<u8>) {
        self.base = state;
        self.log.clear();
        (self.bytes, self.flushed) = (0, 0);
    }
}
