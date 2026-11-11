use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message as Frame;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use yrs::sync::{Message, SyncMessage};
use yrs::updates::decoder::Decode;
use yrs::updates::encoder::Encode;
use yrs::{Doc, ReadTxn, Transact, Update};

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub struct Peer {
    pub doc: Doc,
    socket: Option<Socket>,
    pub closed: Option<u16>,
    pub customs: Vec<(u8, Vec<u8>)>,
    pub presence: Vec<String>,
}

impl Peer {
    pub fn new() -> Peer {
        Peer { doc: api::modules::board::new_doc(), socket: None, closed: None, customs: vec![], presence: vec![] }
    }

    pub async fn connect(&mut self, url: &str, token: Option<&str>) {
        let mut request = url.into_client_request().unwrap();
        if let Some(token) = token {
            request.headers_mut().insert("authorization", format!("Bearer {token}").parse().unwrap());
        }
        let (socket, _) = tokio_tungstenite::connect_async(request).await.unwrap();
        self.socket = Some(socket);
        self.closed = None;
        let sv = self.doc.transact().state_vector();
        self.send(Message::Sync(SyncMessage::SyncStep1(sv))).await;
        self.pump(Duration::from_millis(400)).await;
    }

    pub fn disconnect(&mut self) {
        self.socket = None;
    }

    pub async fn send_raw(&mut self, bytes: Vec<u8>) {
        if let Some(socket) = self.socket.as_mut() {
            _ = socket.send(Frame::Binary(bytes.into())).await;
        }
    }

    pub async fn send(&mut self, message: Message) {
        self.send_raw(message.encode_v1()).await;
    }

    fn on(&mut self, message: Message) -> Option<Message> {
        match message {
            Message::Sync(SyncMessage::SyncStep1(sv)) => {
                Some(Message::Sync(SyncMessage::SyncStep2(self.doc.transact().encode_state_as_update_v1(&sv))))
            }
            Message::Sync(SyncMessage::SyncStep2(u) | SyncMessage::Update(u)) => {
                self.doc.transact_mut().apply_update(Update::decode_v1(&u).unwrap()).unwrap();
                None
            }
            Message::Awareness(update) => {
                self.presence.extend(update.clients.values().map(|entry| entry.json.to_string()));
                None
            }
            Message::Custom(tag, data) => {
                self.customs.push((tag, data));
                None
            }
            _ => None,
        }
    }

    fn closed_with(&mut self, code: u16) {
        self.closed.get_or_insert(code);
        self.socket = None;
    }

    pub async fn pump(&mut self, quiet: Duration) {
        while let Some(socket) = self.socket.as_mut() {
            let Ok(frame) = tokio::time::timeout(quiet, socket.next()).await else { return };
            match frame {
                Some(Ok(Frame::Binary(bytes))) => {
                    let mut decoder = yrs::updates::decoder::DecoderV1::from(bytes.as_ref());
                    let replies: Vec<Message> = yrs::sync::protocol::MessageReader::new(&mut decoder)
                        .filter_map(|m| m.ok())
                        .filter_map(|m| self.on(m))
                        .collect();
                    for reply in replies {
                        self.send(reply).await;
                    }
                }
                Some(Ok(Frame::Close(frame))) => self.closed_with(frame.map_or(1005, |f| u16::from(f.code))),
                Some(Ok(_)) => {}
                Some(Err(_)) | None => self.closed_with(1006),
            }
        }
    }
}
