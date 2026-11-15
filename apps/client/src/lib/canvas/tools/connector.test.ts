import { describe, expect, it } from 'vitest';
import { facing } from './connector';

describe('facing', () => {
	it('binds the side of a spawned shape that faces the connector', () => {
		expect(facing({ x: 0, y: 0 }, { x: 300, y: 40 })).toEqual([0, 0.5]);
		expect(facing({ x: 0, y: 0 }, { x: -300, y: 40 })).toEqual([1, 0.5]);
		expect(facing({ x: 0, y: 0 }, { x: 20, y: 300 })).toEqual([0.5, 0]);
		expect(facing({ x: 0, y: 0 }, { x: 20, y: -300 })).toEqual([0.5, 1]);
	});
});
