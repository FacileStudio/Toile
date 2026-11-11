//! A room's document: the Y.Doc, its validator, and the stored state it can always fall back to.

use std::sync::{Arc, Mutex};

use yrs::updates::decoder::Decode;
use yrs::{Doc, Transact, Update};

use super::command::Edit;
use crate::modules::board::schema::{self, Touched};
use crate::modules::board::{self, DecodeError, SHAPES};

/// Why an update was refused: the close code and reason for its sender.
#[derive(Debug, PartialEq, Eq)]
pub struct Refusal(pub u16, pub String);

/// The document of one room. `base` and `log` are what is stored: the last snapshot and every
/// update accepted since, so a refused update reverts to exactly the state before it.
pub struct Core {
    /// The live document.
    pub doc: Doc,
    touched: Arc<Mutex<Touched>>,
    /// The last snapshot, a full v1 update.
    pub base: Vec<u8>,
    /// Every update accepted since `base`, in order.
    pub log: Vec<Vec<u8>>,
    /// The bytes of `log`.
    pub bytes: usize,
}

impl Core {
    /// The document of `base` then `log`.
    pub fn new(base: Vec<u8>, log: Vec<Vec<u8>>) -> Result<Core, DecodeError> {
        let doc = board::from_state(&base, log.iter().map(Vec::as_slice))?;
        let touched = schema::watch(&doc);
        let bytes = log.iter().map(Vec::len).sum();
        Ok(Core { doc, touched, base, log, bytes })
    }

    fn take(&self) -> Touched {
        std::mem::take(&mut *self.touched.lock().unwrap_or_else(|poisoned| poisoned.into_inner()))
    }

    /// Rebuilds the document from `base` and `log`, dropping anything not accepted.
    pub fn reload(&mut self) -> Result<(), Refusal> {
        let doc = board::from_state(&self.base, self.log.iter().map(Vec::as_slice))
            .map_err(|e| Refusal(1011, format!("revert failed: {e}")))?;
        self.touched = schema::watch(&doc);
        self.doc = doc;
        Ok(())
    }

    fn settle(&mut self, outcome: Result<Vec<u8>, Refusal>) -> Result<Vec<u8>, Refusal> {
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
            Err(refusal) => self.reload().and(Err(refusal)),
        }
    }

    /// Applies a client's `update` and keeps it only when the board still validates; otherwise
    /// the document goes back to the state before it, exactly. Answers whether it held anything.
    pub fn apply(&mut self, update: &[u8]) -> Result<bool, Refusal> {
        if update.len() > schema::MAX_UPDATE {
            return Err(Refusal(1009, "update over 1 MiB".to_owned()));
        }
        let decoded = Update::decode_v1(update).map_err(|e| Refusal(1008, format!("undecodable update: {e}")))?;
        if decoded.is_empty() {
            return Ok(false);
        }
        let applied = self.doc.transact_mut().apply_update(decoded);
        let applied = applied.map_err(|e| Refusal(1008, format!("update refused: {e}")));
        self.settle(applied.map(|()| update.to_vec())).map(|_| true)
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
        self.bytes = 0;
    }
}
