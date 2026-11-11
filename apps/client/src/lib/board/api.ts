import { api } from '$lib/backend';
import type { Board, BoardRole } from '$lib/types';

/** A board as the API answers it. */
export type BoardInfo = Board;

export type { BoardRole };

/** A share link role. */
export type ShareRole = 'view' | 'comment' | 'edit';

/** A share link as listed: the token is shown only once, in the answer that creates it. */
export type Share = { id: string; role: ShareRole; created_at?: string };

/** A share link just created, with its token and its path `/s/<token>`. */
export type NewShare = Share & { token: string; url: string };

/** Where a comment points: a shape and an offset in world units, or a bare board position. */
export type Anchor = { shape?: string; x: number; y: number };

/** A comment row: `mine` says whether the caller wrote it, `guest` whether a link holder did. */
export type Comment = {
	id: string;
	anchor: Anchor;
	body: string;
	parent: string | null;
	author: string;
	guest: boolean;
	mine: boolean;
	resolved_at: string | null;
	created_at: string;
	updated_at: string;
};

/** A new comment, a reply when `parent` is set. */
export type NewComment = { anchor: Anchor; body: string; parent?: string; name?: string };

const scoped = (spaceId: string | null) => (spaceId ? `?${new URLSearchParams({ space_id: spaceId })}` : '');
const json = (method: string, body: unknown) => ({ method, body: JSON.stringify(body) });
const asList = <T>(answer: T[] | { [key: string]: T[] }, key: string): T[] =>
	Array.isArray(answer) ? answer : (answer[key] ?? []);

/** Every board call of the canvas, one method per route of the API contract. */
export const boardApi = {
	list: (spaceId: string | null) => api<BoardInfo[]>(`/boards${scoped(spaceId)}`),
	create: (name: string, spaceId: string | null) =>
		api<BoardInfo>('/boards', json('POST', { name, space_id: spaceId ?? undefined })),
	get: (id: string) => api<BoardInfo>(`/boards/${encodeURIComponent(id)}`),
	rename: (id: string, name: string) => api<BoardInfo>(`/boards/${encodeURIComponent(id)}`, json('PATCH', { name })),
	remove: (id: string) => api<void>(`/boards/${encodeURIComponent(id)}`, { method: 'DELETE' }),
	duplicate: (id: string) => api<BoardInfo>(`/boards/${encodeURIComponent(id)}/duplicate`, { method: 'POST' }),
	shares: (id: string) =>
		api<Share[] | { shares: Share[] }>(`/boards/${encodeURIComponent(id)}/shares`).then((r) => asList(r, 'shares')),
	share: (id: string, role: ShareRole) =>
		api<NewShare>(`/boards/${encodeURIComponent(id)}/shares`, json('POST', { role })),
	revoke: (id: string, share: string) =>
		api<void>(`/boards/${encodeURIComponent(id)}/shares/${encodeURIComponent(share)}`, { method: 'DELETE' }),
	resolve: (token: string) => api<{ board: BoardInfo; role: ShareRole }>(`/share/${encodeURIComponent(token)}`)
};

/** The comment routes of a member (`board`) or a guest (`token`). */
export function commentApi(target: { board: string } | { token: string }) {
	const base =
		'board' in target
			? `/boards/${encodeURIComponent(target.board)}/comments`
			: `/share/${encodeURIComponent(target.token)}/comments`;
	return {
		list: () => api<Comment[] | { comments: Comment[] }>(base).then((r) => asList(r, 'comments')),
		add: (comment: NewComment) => api<Comment>(base, json('POST', comment)),
		resolve: (id: string, resolved: boolean) =>
			api<Comment>(`${base}/${encodeURIComponent(id)}`, json('PATCH', { resolved })),
		remove: (id: string) => api<void>(`${base}/${encodeURIComponent(id)}`, { method: 'DELETE' })
	};
}

/** The role a share link grants, as a board role. */
export function roleOfShare(role: ShareRole): BoardRole {
	return role === 'edit' ? 'editor' : role === 'comment' ? 'commenter' : 'viewer';
}

/** Whether a role may change the document. */
export const canEdit = (role: BoardRole) => role === 'owner' || role === 'editor';

/** Whether a role may post comments. */
export const canComment = (role: BoardRole) => role !== 'viewer';
