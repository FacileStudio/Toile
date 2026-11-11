import { existsSync } from 'node:fs';
import { resolve } from 'node:path';
import { Relay, type Peer } from './relay';

/** A running fake stack: the built client, a stub of the board API and the relay, for client-only checks. */
export type FakeStack = { base: string; relay: Relay; board: string; stop: () => void };

const build = resolve(import.meta.dir, '../build');
const board = {
	id: 'board-1',
	name: 'Tableau test',
	space_id: null,
	created_at: '2026-11-11T10:00:00Z',
	updated_at: '2026-11-11T10:00:00Z'
};
const SHARES: Record<string, 'view' | 'comment' | 'edit'> = {
	'view-token': 'view',
	'comment-token': 'comment',
	'edit-token': 'edit'
};

function person(request: Request): string | null {
	return /(?:^|;\s*)fake=([a-z]+)/.exec(request.headers.get('cookie') ?? '')?.[1] ?? null;
}

function api(path: string, request: Request): Response {
	const who = person(request);
	if (path === '/api/auth/session')
		return who
			? Response.json({ user: { email: `${who}@example.com`, name: who } })
			: new Response(null, { status: 401 });
	if (path === '/api/spaces') return Response.json({ spaces: [] });
	if (path === '/api/boards') return Response.json([{ ...board, role: 'owner' }]);
	if (path === `/api/boards/${board.id}`) return Response.json({ ...board, role: 'owner' });
	if (path.endsWith('/comments')) return Response.json([]);
	const share = /^\/api\/share\/([^/]+)$/.exec(path)?.[1];
	if (share && SHARES[share]) return Response.json({ board: { ...board, role: 'viewer' }, role: SHARES[share] });
	return new Response('not found', { status: 404 });
}

function file(path: string): Response {
	const candidate = resolve(build, `.${path}`);
	if (candidate.startsWith(build) && existsSync(candidate) && !candidate.endsWith('/')) {
		const served = Bun.file(candidate);
		if (served.size) return new Response(served);
	}
	return new Response(Bun.file(resolve(build, 'index.html')));
}

function room(path: string): Peer | null {
	const member = /^\/api\/boards\/([^/]+)\/ws$/.exec(path)?.[1];
	if (member) return { room: member, writable: true, ids: new Set() };
	const token = /^\/api\/share\/([^/]+)\/ws$/.exec(path)?.[1];
	if (token && SHARES[token]) return { room: board.id, writable: SHARES[token] === 'edit', ids: new Set() };
	return null;
}

/** Serves the built client with a stub API and an in-memory relay on a free port. Cookie `fake=<name>` signs in. */
export function startFake(): FakeStack {
	if (!existsSync(`${build}/index.html`)) throw new Error('apps/client/build is missing: run "bun run build" first');
	const relay = new Relay();
	const server = Bun.serve<Peer>({
		port: 0,
		fetch(request, srv) {
			const path = new URL(request.url).pathname;
			const peer = room(path);
			if (peer && srv.upgrade(request, { data: peer })) return undefined;
			return path.startsWith('/api/') ? api(path, request) : file(path);
		},
		websocket: {
			open: (socket) => relay.open(socket),
			message: (socket, data) =>
				relay.message(socket, typeof data === 'string' ? new TextEncoder().encode(data) : new Uint8Array(data)),
			close: (socket) => relay.close(socket)
		}
	});
	return { base: `http://127.0.0.1:${server.port}`, relay, board: board.id, stop: () => server.stop(true) };
}

if (import.meta.main) {
	const stack = startFake();
	console.log(`fake stack on ${stack.base} (cookie fake=alice to sign in; board ${stack.board})`);
}
