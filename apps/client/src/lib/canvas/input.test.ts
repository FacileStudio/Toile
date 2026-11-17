import { describe, expect, it } from 'vitest';
import { wheelFactor } from './input';

describe('input', () => {
	it('zooms in on a negative wheel delta, finer for a mouse than a pinch', () => {
		expect(wheelFactor(-100, 0, false)).toBeGreaterThan(1);
		expect(wheelFactor(100, 0, true)).toBeLessThan(wheelFactor(100, 0, false));
		expect(wheelFactor(3, 1, false)).toBeCloseTo(wheelFactor(48, 0, false));
	});
});
