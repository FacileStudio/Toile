import { t, type AppKey } from '$lib/i18n/index.svelte';
import { openRoom as openSalon, socketUrl, type Connection, type Room, type Stop } from '@facile/salon';
import type * as Y from 'yjs';
import { apiOrigin, socketProtocols } from '$lib/platform';

export type { Connection, Room, Stop };

const STOP_TEXT: Record<Stop, AppKey> = {
	refused: 'stop.refused',
	'too-large': 'stop.tooLarge',
	failed: 'stop.failed'
};

const REASONS: Record<string, AppKey> = {
	'access revoked': 'stop.revoked',
	'board deleted': 'stop.deleted'
};

/** What the stop banner says for a terminal close: the known reasons in words, any other with the server's detail. */
export function stopMessage(stop: Stop, reason: string): string {
	const known = REASONS[reason.trim()];
	if (known) return t(known);
	const detail = reason.trim();
	if (stop !== 'refused' || !detail) return t(STOP_TEXT[stop]);
	return t('stop.detail', { refused: t('stop.refused'), detail });
}

/** Where a board's room lives: a member's board id, or a guest's share token and display name. */
export type RoomTarget = { board: string } | { token: string; name: string };

/** The y-websocket base URL and room name for a target on this origin. */
export function roomUrl(target: RoomTarget, location: Pick<Location, 'protocol' | 'host'>): [string, string] {
	if ('board' in target) return [socketUrl('/api/boards', location), `${encodeURIComponent(target.board)}/ws`];
	return [socketUrl('/api/share', location), `${encodeURIComponent(target.token)}/ws`];
}

/** What a page hears from its room. */
export type RoomEvents = {
	onconnection: (state: Connection) => void;
	onstop: (stop: Stop, reason: string) => void;
	onsynced?: () => void;
	oncomments?: () => void;
};

function isComments(notice: unknown): boolean {
	return notice === null || (notice as { type?: string }).type === 'comments-changed';
}

/** Opens the room of `target` over salon: stop on 1008/1009/1011, else reconnect; a comments notice refreshes comments. */
export function openRoom(target: RoomTarget, ydoc: Y.Doc, events: RoomEvents): Room {
	const origin = apiOrigin();
	const [base, room] = roomUrl(target, origin ? new URL(origin) : window.location);
	return openSalon(
		{ base, room, protocols: socketProtocols(), params: 'token' in target ? { name: target.name } : {} },
		ydoc,
		{
			onconnection: events.onconnection,
			onstop: events.onstop,
			onsynced: events.onsynced,
			oncustom: (notice) => {
				if (isComments(notice)) events.oncomments?.();
			}
		}
	);
}
