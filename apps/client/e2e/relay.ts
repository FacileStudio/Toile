import type { ServerWebSocket } from 'bun';
import * as decoding from 'lib0/decoding';
import * as encoding from 'lib0/encoding';
import * as awarenessProtocol from 'y-protocols/awareness';
import * as syncProtocol from 'y-protocols/sync';
import * as Y from 'yjs';

/** What a socket of the relay carries: its room and whether it may change the document. */
export type Peer = { room: string; writable: boolean; ids: Set<number> };

type Room = { doc: Y.Doc; awareness: awarenessProtocol.Awareness; sockets: Set<ServerWebSocket<Peer>> };

const SYNC = 0;
const AWARENESS = 1;

function send(socket: ServerWebSocket<Peer>, encoder: encoding.Encoder): void {
	if (encoding.length(encoder) > 1) socket.send(encoding.toUint8Array(encoder));
}

/**
 * A tiny in-memory y-websocket relay for tests and the perf script, never shipped: one Y.Doc and one awareness per
 * room, sync and awareness messages only, and read-only sockets whose updates it drops like the real room does.
 */
export class Relay {
	private rooms = new Map<string, Room>();

	/** The room `name`, created empty on first use. */
	room(name: string): Room {
		let room = this.rooms.get(name);
		if (room) return room;
		const doc = new Y.Doc();
		const awareness = new awarenessProtocol.Awareness(doc);
		awareness.setLocalState(null);
		room = { doc, awareness, sockets: new Set() };
		const created = room;
		doc.on('update', (update: Uint8Array) => {
			const encoder = encoding.createEncoder();
			encoding.writeVarUint(encoder, SYNC);
			syncProtocol.writeUpdate(encoder, update);
			for (const socket of created.sockets) send(socket, encoder);
		});
		awareness.on('update', ({ added, updated, removed }: Record<string, number[]>, origin: unknown) => {
			if (origin && typeof origin === 'object' && 'data' in origin)
				for (const id of added) (origin as ServerWebSocket<Peer>).data.ids.add(id);
			const encoder = encoding.createEncoder();
			encoding.writeVarUint(encoder, AWARENESS);
			encoding.writeVarUint8Array(
				encoder,
				awarenessProtocol.encodeAwarenessUpdate(awareness, [...added, ...updated, ...removed])
			);
			for (const socket of created.sockets) send(socket, encoder);
		});
		this.rooms.set(name, room);
		return room;
	}

	/** Greets a new socket with sync step 1 and the current awareness. */
	open(socket: ServerWebSocket<Peer>): void {
		const room = this.room(socket.data.room);
		room.sockets.add(socket);
		const encoder = encoding.createEncoder();
		encoding.writeVarUint(encoder, SYNC);
		syncProtocol.writeSyncStep1(encoder, room.doc);
		send(socket, encoder);
		const states = [...room.awareness.getStates().keys()];
		if (!states.length) return;
		const aware = encoding.createEncoder();
		encoding.writeVarUint(aware, AWARENESS);
		encoding.writeVarUint8Array(aware, awarenessProtocol.encodeAwarenessUpdate(room.awareness, states));
		send(socket, aware);
	}

	/** Handles one binary message from a socket. */
	message(socket: ServerWebSocket<Peer>, data: Uint8Array): void {
		const room = this.room(socket.data.room);
		const decoder = decoding.createDecoder(data);
		const type = decoding.readVarUint(decoder);
		if (type === AWARENESS) {
			awarenessProtocol.applyAwarenessUpdate(room.awareness, decoding.readVarUint8Array(decoder), socket);
			return;
		}
		if (type !== SYNC) return;
		const encoder = encoding.createEncoder();
		encoding.writeVarUint(encoder, SYNC);
		const kind = decoding.peekVarUint(decoder);
		if (kind !== syncProtocol.messageYjsSyncStep1 && !socket.data.writable) return;
		syncProtocol.readSyncMessage(decoder, encoder, room.doc, socket);
		send(socket, encoder);
	}

	/** Forgets a closed socket and its awareness state. */
	close(socket: ServerWebSocket<Peer>): void {
		const room = this.room(socket.data.room);
		room.sockets.delete(socket);
		awarenessProtocol.removeAwarenessStates(room.awareness, [...socket.data.ids], null);
	}

	/** Sends the custom comments-changed message (type 100) to every socket of room `name`. */
	commentsChanged(name: string): void {
		const encoder = encoding.createEncoder();
		encoding.writeVarUint(encoder, 100);
		encoding.writeVarString(encoder, JSON.stringify({ type: 'comments-changed' }));
		for (const socket of this.room(name).sockets) send(socket, encoder);
	}
}
