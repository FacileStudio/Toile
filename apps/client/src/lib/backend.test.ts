import { afterEach, describe, expect, it, vi } from 'vitest';

vi.mock('$app/environment', () => ({ browser: true }));
vi.mock('$app/navigation', () => ({ goto: async () => undefined }));

const { backend } = await import('./backend');

function stub(response: Response) {
	const fetch = vi.fn(async () => response);
	vi.stubGlobal('fetch', fetch);
	return fetch;
}

const sent = (fetch: ReturnType<typeof stub>) => fetch.mock.calls[0] as unknown as [string, RequestInit];

afterEach(() => vi.unstubAllGlobals());

describe('backend', () => {
	it('lists the personal scope without a query and a space with space_id', async () => {
		const fetch = stub(Response.json([]));
		await backend.boards(null);
		expect(sent(fetch)[0]).toBe('/api/boards');
		fetch.mockResolvedValueOnce(Response.json([]));
		await backend.boards('space a');
		expect((fetch.mock.calls[1] as unknown as [string])[0]).toBe('/api/boards?space_id=space+a');
	});

	it('creates a board with the CSRF header and a JSON body', async () => {
		const fetch = stub(Response.json({ id: 'b1', name: 'Atelier' }, { status: 201 }));
		const board = await backend.createBoard({ name: 'Atelier' });
		const [url, init] = sent(fetch);
		expect(url).toBe('/api/boards');
		expect(init.method).toBe('POST');
		expect(new Headers(init.headers).get('X-Facile-CSRF')).toBe('1');
		expect(JSON.parse(String(init.body))).toEqual({ name: 'Atelier' });
		expect(board.id).toBe('b1');
	});

	it('throws the suite envelope as an ApiError', async () => {
		stub(Response.json({ error: { code: 'not_found', message: 'space not found' } }, { status: 404 }));
		await expect(backend.boards('gone')).rejects.toMatchObject({ status: 404, code: 'not_found' });
	});
});
