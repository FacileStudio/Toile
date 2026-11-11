import { afterEach, describe, expect, it, vi } from 'vitest';

vi.mock('$app/environment', () => ({ browser: true }));
vi.mock('$app/navigation', () => ({ goto: async () => undefined }));

const { assetUrl, uploadImage } = await import('./assets');
const { resolveCard } = await import('./cards');

afterEach(() => vi.unstubAllGlobals());

describe('assets and cards', () => {
	it('uploads the file as multipart with the CSRF header and no JSON content type', async () => {
		const answer = { asset: 'a1', url: '/api/boards/b1/assets/a1', w: 3, h: 2 };
		const fetch = vi.fn(async () => Response.json(answer, { status: 201 }));
		vi.stubGlobal('fetch', fetch);
		const result = await uploadImage('b1', new File([new Uint8Array([1])], 'x.png', { type: 'image/png' }));
		const [url, init] = fetch.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toBe('/api/boards/b1/assets');
		expect(init.body).toBeInstanceOf(FormData);
		expect(new Headers(init.headers).get('Content-Type')).toBeNull();
		expect(new Headers(init.headers).get('X-Facile-CSRF')).toBe('1');
		expect(result).toEqual(answer);
	});

	it('reads members through the board and guests through their link', () => {
		expect(assetUrl('b1', 'a1')).toBe('/api/boards/b1/assets/a1');
		expect(assetUrl('b1', 'a1', 't/k')).toBe('/api/share/t%2Fk/assets/a1');
	});

	it('resolves a card with a JSON body', async () => {
		const fetch = vi.fn(async () => Response.json({ kind: 'link', title: 'x.test', icon: 'link', url: 'https://x.test/' }));
		vi.stubGlobal('fetch', fetch);
		expect((await resolveCard('https://x.test/')).title).toBe('x.test');
		const [, init] = fetch.mock.calls[0] as unknown as [string, RequestInit];
		expect(JSON.parse(String(init.body))).toEqual({ url: 'https://x.test/' });
	});
});
