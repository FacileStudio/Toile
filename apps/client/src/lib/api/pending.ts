import { ApiError } from '@facile/client';
import { api } from '$lib/backend';

/** An agent action waiting for the person, one of `GET /api/mcp/pending`'s `actions`. */
export type PendingAction = {
	id: string;
	kind: string;
	target: string;
	summary: string;
	client_id: string;
	created_at: string;
	expires_at: string;
	state: string;
};

/** How the confirm page describes an action's state: ready, or its shapes changed or vanished since. */
export const STATES: Record<string, { tone: 'success' | 'warning' | 'danger'; label: string }> = {
	ready: { tone: 'success', label: 'Prête' },
	changed: { tone: 'warning', label: 'Le tableau a changé depuis la demande : la confirmation échouera' },
	missing: { tone: 'danger', label: 'Le tableau n’existe plus ou vous n’y avez plus accès' }
};

/** The board an action touches, read from its target (`{"board_id", …}`), or null. */
export function targetBoard(action: Pick<PendingAction, 'target'>): string | null {
	try {
		const target = JSON.parse(action.target) as { board_id?: unknown };
		return typeof target.board_id === 'string' ? target.board_id : null;
	} catch {
		return null;
	}
}

/** tronc's refusal of anything but the person's own browser session on the confirm page. */
export const OWN_SESSION_REFUSAL = "only the person's own session can confirm actions";

/**
 * Whether `error` is the confirm page's refusal of a session that is not the person's own browser
 * login: a cookie from before the upgrade, a CLI or API token, or the mobile or desktop app.
 * Signing in again in the browser fixes it.
 */
export function notOwnSession(error: unknown): boolean {
	return (
		error instanceof ApiError &&
		error.status === 403 &&
		error.code === 'permission_denied' &&
		error.message === OWN_SESSION_REFUSAL
	);
}

const path = (id: string, verb: string) => `/mcp/pending/${encodeURIComponent(id)}/${verb}`;

/** The confirm page's calls. */
export const pending = {
	list: () => api<{ actions: PendingAction[]; truncated?: boolean }>('/mcp/pending').then((r) => r.actions),
	confirm: (id: string) => api<{ status: string }>(path(id, 'confirm'), { method: 'POST' }),
	dismiss: (id: string) => api<{ status: string }>(path(id, 'dismiss'), { method: 'POST' })
};
