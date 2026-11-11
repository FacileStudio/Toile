use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

/// A frame for one connection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Out {
    /// A y-websocket message.
    Binary(Vec<u8>),
    /// Close the socket with this code and reason.
    Close(u16, String),
}

/// The room unloaded while the command was in flight; the hub retries on a fresh room.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gone;

/// A command answer: `Err(Gone)` asks for a retry, the inner result is the room's.
pub type Reply<T> = oneshot::Sender<Result<tronc::Result<T>, Gone>>;

/// A server-side change to a board: runs in one Yjs transaction over `shapes` and is validated
/// like any client update.
pub type Edit = Box<dyn FnOnce(&mut yrs::TransactionMut, &yrs::MapRef) -> Result<(), String> + Send>;

/// Who a connection speaks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Who {
    /// The porte user; `None` for a guest.
    pub user_id: Option<i64>,
    /// The share link a guest came through.
    pub share: Option<Uuid>,
    /// The name shown in presence; a member's awareness `user.name` is overwritten with it.
    pub name: String,
    /// Whether its document updates are applied; viewers and commenters send presence only.
    pub writer: bool,
}

/// A connection joining a room.
#[derive(Debug, Clone)]
pub struct Member {
    /// The connection.
    pub conn: u64,
    /// Who it speaks for.
    pub who: Who,
    /// Where the room writes to it.
    pub outbox: mpsc::Sender<Out>,
}

/// One instruction to a room task.
pub enum Command {
    /// A connection joins and receives the opening sync messages.
    Join(Member, Reply<()>),
    /// A binary frame from a connection.
    Message(u64, Vec<u8>),
    /// A connection left.
    Leave(u64),
    /// The board's whole state, as one Yjs v1 update.
    Read(Reply<Vec<u8>>),
    /// A server-side change, validated, broadcast and stored before the answer.
    Write(Edit, Reply<()>),
    /// A message for every member, such as the comments notice.
    Broadcast(Vec<u8>),
    /// Close every connection of these users that lost access to the board.
    Evict(Vec<i64>),
    /// Close every connection that came through this share link.
    Revoke(Uuid),
    /// Persist now, close every member with this code and reason, and unload.
    Shutdown(u16, String, oneshot::Sender<()>),
}
