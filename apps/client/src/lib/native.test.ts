import { afterEach, describe, expect, it, vi } from 'vitest';
import { cleanCode, finishLoopback, loginUrl, randomState } from './native';

afterEach(() => vi.unstubAllGlobals());

describe('native sign-in', () => {
	it('builds a state porte keeps on the CLI flow', () => {
		const state = randomState();
		expect(state).toMatch(/^[A-Za-z0-9_-]{32}$/);
		expect(randomState()).not.toBe(state);
	});

	it('asks porte for the loopback redirect only with a port', () => {
		expect(loginUrl('s1', 50123)).toBe('/api/auth/oidc?flow=cli&cli_state=s1&port=50123');
		expect(loginUrl('s1')).toBe('/api/auth/oidc?flow=cli&cli_state=s1');
	});

	it('cleans a pasted code and refuses anything else', () => {
		expect(cleanCode('  abcDEF012345_-xyz\n')).toBe('abcDEF012345_-xyz');
		expect(cleanCode('abc')).toBe('');
		expect(cleanCode('<script>alert(1)</script>')).toBe('');
	});

	it('refuses a loopback answer for another state without calling the API', async () => {
		const fetch = vi.fn();
		vi.stubGlobal('fetch', fetch);
		await expect(finishLoopback({ code: 'c', state: 'other', error: '' }, 'mine')).rejects.toThrow();
		await expect(finishLoopback({ code: '', state: 'mine', error: 'refusé' }, 'mine')).rejects.toThrow('refusé');
		expect(fetch).not.toHaveBeenCalled();
	});

	it('redeems the code of a matching answer', async () => {
		const fetch = vi.fn(async () => Response.json({ user_id: '1', token: 't' }));
		vi.stubGlobal('fetch', fetch);
		await finishLoopback({ code: 'c0de', state: 'mine', error: '' }, 'mine');
		const [url, init] = fetch.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toBe('/api/auth/oidc/exchange');
		expect(init.body).toBe(JSON.stringify({ code: 'c0de' }));
	});
});
