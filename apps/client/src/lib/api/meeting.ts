import { api } from '$lib/backend';
import type { Board } from '$lib/types';

/** What `POST /api/boards/meeting` answers: the room's board and its `/s/<token>` edit link. */
export type Meeting = { board: Board; edit_link: string; created: boolean };

/** The meeting board of an Echo room for the signed-in person, made on the first call. */
export function meetingBoard(room: string, options: { name?: string; space_id?: string } = {}): Promise<Meeting> {
	return api<Meeting>('/boards/meeting', { method: 'POST', body: JSON.stringify({ room, ...options }) });
}
