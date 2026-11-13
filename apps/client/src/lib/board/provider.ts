import * as decoding from 'lib0/decoding';
import { WebsocketProvider } from 'y-websocket';
import type * as Y from 'yjs';
import { apiOrigin, socketProtocols } from '$lib/platform';

/** What the connection chip shows. */
export type Connection = 'connecting' | 'live' | 'offline' | 'stopped';

/** Why the server ended the session for good, from its close code. */
export type Stop = 'refused' | 'too-large' | 'failed';

/** The custom y-websocket message type the room uses to say the comments changed. */
export const MESSAGE_COMMENTS = 100;

const STOPS: Record<number, Stop> = { 1008: 'refused', 1009: 'too-large', 1011: 'failed' };

/** The stop a close code means, or null to reconnect with backoff (1001 room closed, 1013 restart, network). */
export function stopOf(code: number): Stop | null {
	return STOPS[code] ?? null;
}

const STOP_TEXT: Record<Stop, string> = {
	refused: 'Le serveur a refusé une modification.',
	'too-large': 'La dernière modification est trop volumineuse.',
	failed: 'Le tableau a rencontré une erreur. Rechargez la page.'
};

const REASONS: Record<string, string> = {
	'access revoked': 'Votre accès à ce tableau a été retiré.',
	'board deleted': 'Ce tableau a été supprimé.'
};

/** What the stop banner says for a terminal close: the known reasons in words, any other with the server's detail. */
export function stopMessage(stop: Stop, reason: string): string {
	const known = REASONS[reason.trim()];
	if (known) return known;
	const detail = reason.trim();
	if (stop !== 'refused' || !detail) return STOP_TEXT[stop];
	return `${STOP_TEXT.refused} Rechargez la page. Détail : ${detail}`;
}

/** Where a board's room lives: a member's board id, or a guest's share token and display name. */
export type RoomTarget = { board: string } | { token: string; name: string };

/** The y-websocket base URL and room name for a target on this origin. */
export function roomUrl(target: RoomTarget, location: Pick<Location, 'protocol' | 'host'>): [string, string] {
	const scheme = location.protocol === 'https:' ? 'wss:' : 'ws:';
	if ('board' in target) return [`${scheme}//${location.host}/api/boards`, `${encodeURIComponent(target.board)}/ws`];
	return [`${scheme}//${location.host}/api/share`, `${encodeURIComponent(target.token)}/ws`];
}

/** The connection state from the provider status, the first sync and the browser's network flag. */
export function connectionOf(status: string, synced: boolean, online: boolean): Connection {
	if (!online) return 'offline';
	return status === 'connected' && synced ? 'live' : 'connecting';
}

/** What a page hears from its room. */
export type RoomEvents = {
	onconnection: (state: Connection) => void;
	onstop: (stop: Stop, reason: string) => void;
	onsynced?: () => void;
	oncomments?: () => void;
};

/** An open room: the provider (awareness at `provider.awareness`) and how to close it. */
export type Room = { provider: WebsocketProvider; destroy: () => void };

function commentsHandler(events: RoomEvents) {
	return (_encoder: unknown, decoder: decoding.Decoder) => {
		try {
			const message = JSON.parse(decoding.readVarString(decoder)) as { type?: string };
			if (message.type === 'comments-changed') events.oncomments?.();
		} catch {
			events.oncomments?.();
		}
	};
}

/** Opens the room of `target` over y-websocket 3: no BroadcastChannel, stop on 1008/1009/1011, else reconnect. */
export function openRoom(target: RoomTarget, ydoc: Y.Doc, events: RoomEvents): Room {
	const origin = apiOrigin();
	const [base, name] = roomUrl(target, origin ? new URL(origin) : window.location);
	const provider = new WebsocketProvider(base, name, ydoc, {
		disableBc: true,
		protocols: socketProtocols(),
		params: 'token' in target ? { name: target.name } : {},
		shouldReconnect: (event) => stopOf(event.code) === null
	});
	provider.messageHandlers[MESSAGE_COMMENTS] = commentsHandler(events) as (typeof provider.messageHandlers)[number];
	let status = 'connecting';
	let stopped = false;
	const report = () =>
		events.onconnection(stopped ? 'stopped' : connectionOf(status, provider.synced, navigator.onLine));
	provider.on('status', (event: { status: string }) => {
		status = event.status;
		report();
	});
	provider.on('sync', (synced: boolean) => {
		if (synced) events.onsynced?.();
		report();
	});
	provider.on('closed', (event: { code: number; reason: string }) => {
		const stop = stopOf(event.code);
		if (!stop) return;
		stopped = true;
		report();
		events.onstop(stop, event.reason);
	});
	window.addEventListener('online', report);
	window.addEventListener('offline', report);
	report();
	return {
		provider,
		destroy: () => {
			window.removeEventListener('online', report);
			window.removeEventListener('offline', report);
			provider.destroy();
		}
	};
}
