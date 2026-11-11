import { createApi, type CallOptions } from '@facile/client';
import { isReconnect, reconnect } from './reconnect.svelte';
import type { Board, Entitlements, Space } from './types';

const kernel = createApi();

/**
 * The API kernel: same origin, cookies, the CSRF header on mutations, the suite envelope thrown
 * typed. A 412 from a token exchange also asks the person to sign in again.
 */
export async function api<T>(path: string, init?: RequestInit, options?: CallOptions): Promise<T> {
	try {
		return await kernel<T>(path, init, options);
	} catch (error) {
		if (isReconnect(error)) reconnect.needed = true;
		throw error;
	}
}

const scoped = (spaceId: string | null) => (spaceId ? `?${new URLSearchParams({ space_id: spaceId })}` : '');

/** Every call the client makes, one method per route of `docs/api.md`. */
export const backend = {
	loginUrl: () => '/api/auth/oidc',
	spaces: () => api<{ spaces: Space[] }>('/spaces').then((r) => r.spaces),
	boards: (spaceId: string | null) => api<Board[]>(`/boards${scoped(spaceId)}`),
	createBoard: (input: { name: string; space_id?: string }) =>
		api<Board>('/boards', { method: 'POST', body: JSON.stringify(input) }),
	entitlements: (spaceId: string | null) => api<Entitlements>(`/me/entitlements${scoped(spaceId)}`)
};
