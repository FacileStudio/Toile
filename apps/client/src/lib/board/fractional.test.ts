import { describe, expect, it } from 'vitest';
import { compareZ, keyBetween, keysBetween } from './fractional';

function randomKeys(count: number): string[] {
	const keys: string[] = [];
	for (let i = 0; i < count; i += 1) {
		const at = Math.floor(Math.random() * (keys.length + 1));
		keys.splice(at, 0, keyBetween(keys[at - 1] ?? null, keys[at] ?? null));
	}
	return keys;
}

describe('fractional index', () => {
	it('puts a key strictly between its neighbours, at either open end', () => {
		const a = keyBetween(null, null);
		const b = keyBetween(a, null);
		const c = keyBetween(null, a);
		const d = keyBetween(a, b);
		expect([c, a, d, b]).toEqual([c, a, d, b].slice().sort());
		expect(new Set([a, b, c, d]).size).toBe(4);
	});

	it('keeps every key ordered under 2,000 random inserts and never ends a key in 0', () => {
		const keys = randomKeys(2000);
		expect(keys).toEqual(keys.slice().sort());
		expect(new Set(keys).size).toBe(keys.length);
		expect(keys.some((key) => key.endsWith('0'))).toBe(false);
	});

	it('fits keys between adjacent digits and under the lowest key', () => {
		expect(keyBetween('1', '2') > '1' && keyBetween('1', '2') < '2').toBe(true);
		const low = keyBetween(null, '01');
		expect(low < '01').toBe(true);
		expect(keyBetween('', '1') < '1').toBe(true);
	});

	it('grows bulk keys with log(n), not n', () => {
		const keys = keysBetween(null, null, 5000);
		expect(keys).toEqual(keys.slice().sort());
		expect(Math.max(...keys.map((k) => k.length))).toBeLessThan(8);
		const appended = [keyBetween(null, null)];
		for (let i = 0; i < 1000; i += 1) appended.push(keyBetween(appended[appended.length - 1], null));
		expect(appended[appended.length - 1].length).toBeLessThan(40);
	});

	it('answers a misordered pair with a key after the first, and breaks z ties by id', () => {
		expect(keyBetween('b', 'a') > 'b').toBe(true);
		expect(compareZ({ z: 'V', id: 'a' }, { z: 'V', id: 'b' })).toBe(-1);
		expect(compareZ({ z: 'W', id: 'a' }, { z: 'V', id: 'b' })).toBe(1);
	});
});
