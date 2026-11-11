import { api } from '$lib/backend';

/** An agent action waiting for the person, as `GET /api/mcp/actions` answers it. */
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

const path = (id: string, verb: string) => `/mcp/actions/${encodeURIComponent(id)}/${verb}`;

/** The confirm page's calls. */
export const pending = {
	list: () => api<PendingAction[]>('/mcp/actions'),
	confirm: (id: string) => api<{ status: string }>(path(id, 'confirm'), { method: 'POST' }),
	dismiss: (id: string) => api<void>(path(id, 'dismiss'), { method: 'POST' })
};
