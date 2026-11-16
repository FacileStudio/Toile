//! A board as salon's room document: member updates checked against schema 1, exact revert.

use salon::{Refusal, Space};
use yrs::updates::decoder::Decode;
use yrs::{Doc, Transact, Update};

use super::core::Core;
use crate::modules::board::{self, schema};

impl Space for Core {
    fn doc(&self) -> &Doc {
        &self.doc
    }

    fn apply(&mut self, update: &[u8]) -> Result<bool, Refusal> {
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

    fn recover(&mut self) -> Result<(), Refusal> {
        let doc = board::from_state(&self.base, self.log.iter().map(Vec::as_slice))
            .map_err(|e| Refusal(1011, format!("revert failed: {e}")))?;
        self.touched = schema::watch(&doc);
        self.doc = doc;
        Ok(())
    }

    fn backlog(&self) -> (usize, usize) {
        (self.log.len(), self.bytes)
    }
}
