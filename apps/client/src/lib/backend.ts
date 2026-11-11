import { createApi } from '@facile/client';
import type { Board, Space } from './types';

/** The API kernel: same origin, cookies, the CSRF header on mutations, the suite envelope thrown typed. */
export const api = createApi();

const scoped = (spaceId: string | null) => (spaceId ? `?${new URLSearchParams({ space_id: spaceId })}` : '');

/** Every call the client makes, one method per route of `docs/api.md`. */
export const backend = {
	loginUrl: () => '/api/auth/oidc',
	spaces: () => api<{ spaces: Space[] }>('/spaces').then((r) => r.spaces),
	boards: (spaceId: string | null) => api<Board[]>(`/boards${scoped(spaceId)}`),
	createBoard: (input: { name: string; space_id?: string }) =>
		api<Board>('/boards', { method: 'POST', body: JSON.stringify(input) })
};
