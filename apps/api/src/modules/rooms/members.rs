use tokio::time::Instant;

use super::command::{Command, Member, Out};
use super::room::{Conn, Room};
use super::sync::{Effect, guarded, opening};
use crate::modules::boards::access::{self, Access};

impl Room {
    /// Handles one command; `false` ends the room.
    pub(super) async fn handle(&mut self, command: Command) -> bool {
        match command {
            Command::Join(member, reply) => {
                self.join(member);
                _ = reply.send(Ok(Ok(())));
            }
            Command::Message(conn, frame) => self.message(conn, &frame),
            Command::Leave(conn) => self.leave(conn).await,
            Command::Read(reply) => _ = reply.send(Ok(Ok(self.core.state()))),
            Command::Write(edit, reply) => _ = reply.send(Ok(self.write(edit).await)),
            Command::Broadcast(frame) => self.broadcast(None, &frame),
            Command::Evict(users) => self.evict(&users).await,
            Command::Revoke(share) => self.close_where(|conn| conn.who.share == Some(share)),
            Command::Shutdown(code, reason, done) => {
                self.stop(code, &reason).await;
                _ = done.send(());
                return false;
            }
        }
        true
    }

    fn join(&mut self, member: Member) {
        let conn = Conn { outbox: member.outbox, who: member.who, clients: Default::default() };
        self.conns.insert(member.conn, conn);
        self.empty_since = None;
        for frame in opening(&self.core, &self.awareness) {
            self.send(member.conn, Out::Binary(frame));
        }
    }

    fn message(&mut self, conn: u64, frame: &[u8]) {
        let Some(writer) = self.conns.get(&conn).map(|c| c.who.writer) else { return };
        for effect in guarded(&mut self.core, &self.awareness, frame, writer) {
            match effect {
                Effect::Reply(bytes) => self.send(conn, Out::Binary(bytes)),
                Effect::Accepted(bytes) => {
                    self.broadcast(Some(conn), &bytes);
                    self.accepted();
                }
                Effect::Presence(update) => self.presence(conn, update),
                Effect::Refuse(code, reason) => self.drop_conn(conn, Some(Out::Close(code, reason))),
            }
        }
    }

    async fn leave(&mut self, conn: u64) {
        self.drop_conn(conn, None);
        if self.conns.is_empty() {
            self.snapshot().await;
        }
    }

    /// Removes a member, tells the others its cursors left, and closes it with `close`.
    pub(super) fn drop_conn(&mut self, conn: u64, close: Option<Out>) {
        let Some(member) = self.conns.remove(&conn) else { return };
        if let Some(close) = close {
            _ = member.outbox.try_send(close);
        }
        if let Some(gone) = super::presence::departure(&mut self.awareness, &member.clients) {
            self.broadcast(None, &gone);
        }
        if self.conns.is_empty() {
            self.empty_since = Some(Instant::now());
        }
    }

    /// Sends `frame` to every member but `except`.
    pub(super) fn broadcast(&mut self, except: Option<u64>, frame: &[u8]) {
        let targets: Vec<u64> = self.conns.keys().copied().filter(|c| Some(*c) != except).collect();
        for conn in targets {
            self.send(conn, Out::Binary(frame.to_vec()));
        }
    }

    async fn evict(&mut self, users: &[i64]) {
        let mut lost = Vec::new();
        for user in users.iter().filter(|u| self.conns.values().any(|c| c.who.user_id == Some(**u))) {
            if matches!(access::access(&self.state, *user, self.id).await, Ok(Access::None)) {
                lost.push(*user);
            }
        }
        self.close_where(|conn| conn.who.user_id.is_some_and(|user| lost.contains(&user)));
    }

    /// Closes every member `pick` selects with 1008: its access was revoked.
    fn close_where(&mut self, pick: impl Fn(&Conn) -> bool) {
        let gone: Vec<u64> = self.conns.iter().filter(|(_, c)| pick(c)).map(|(id, _)| *id).collect();
        for conn in gone {
            self.drop_conn(conn, Some(Out::Close(1008, "access revoked".to_owned())));
        }
    }
}
