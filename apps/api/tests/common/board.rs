use yrs::sync::{Message, SyncMessage};
use yrs::{Map, MapPrelim, MapRef, Transact};

use super::ysync::Peer;

impl Peer {
    pub async fn announce(&mut self, state: &str) {
        let mut awareness = yrs::sync::Awareness::new(self.doc.clone());
        awareness.set_local_state_raw(state);
        self.send(Message::Awareness(awareness.update().unwrap())).await;
    }

    pub fn update(&self, change: impl FnOnce(&mut yrs::TransactionMut, &MapRef)) -> Vec<u8> {
        let shapes = self.doc.get_or_insert_map("shapes");
        let mut txn = self.doc.transact_mut();
        change(&mut txn, &shapes);
        txn.encode_update_v1()
    }

    pub async fn edit(&mut self, change: impl FnOnce(&mut yrs::TransactionMut, &MapRef)) {
        let update = self.update(change);
        self.send(Message::Sync(SyncMessage::Update(update))).await;
    }

    pub fn shapes(&self) -> Vec<String> {
        let shapes = self.doc.get_or_insert_map("shapes");
        let txn = self.doc.transact();
        let mut ids: Vec<String> = shapes.keys(&txn).map(str::to_owned).collect();
        ids.sort();
        ids
    }
}

pub fn rect(txn: &mut yrs::TransactionMut, shapes: &MapRef, id: &str, x: f64) {
    let fields: Vec<(&str, yrs::Any)> = vec![("type", "rect".into()), ("x", x.into()), ("y", 0.0.into())];
    shapes.insert(txn, id, fields.into_iter().collect::<MapPrelim>());
}

pub fn guest(base: &str, token: &str, name: &str) -> String {
    format!("{base}/api/share/{token}/ws?name={name}")
}

pub fn ws(base: &str, board: &str) -> String {
    format!("{base}/api/boards/{board}/ws")
}
