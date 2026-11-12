import { describe, expect, it } from 'vitest';
import { CORNERS, EDGES, HANDLES, resizeBox } from './transform';

describe('handle lists', () => {
	it('exports four corners and four edges matching the full list', () => {
		expect(CORNERS).toEqual(['nw', 'ne', 'se', 'sw']);
		expect(EDGES).toEqual(['n', 'e', 's', 'w']);
		expect([...CORNERS, ...EDGES]).toEqual(HANDLES);
	});
});

describe('resizeBox with corners and aspect ratio', () => {
	it('preserves square proportions when resizing from southeast', () => {
		const box = { x: 0, y: 0, w: 100, h: 100 };
		const result = resizeBox(box, 'se', 50, 20, true);
		expect(result).toEqual({ x: 0, y: 0, w: 150, h: 150 });
	});

	it('preserves square proportions when resizing from northwest', () => {
		const box = { x: 100, y: 100, w: 100, h: 100 };
		const result = resizeBox(box, 'nw', -40, -10, true);
		expect(result).toEqual({ x: 60, y: 60, w: 140, h: 140 });
	});
});

describe('resizeBox with edges', () => {
	it('resizes width only without locking aspect ratio', () => {
		const box = { x: 0, y: 0, w: 100, h: 100 };
		const east = resizeBox(box, 'e', 50, 0, true);
		expect(east).toEqual({ x: 0, y: 0, w: 150, h: 100 });
		const west = resizeBox(box, 'w', 20, 0, true);
		expect(west).toEqual({ x: 20, y: 0, w: 80, h: 100 });
	});

	it('resizes height only without locking aspect ratio', () => {
		const box = { x: 0, y: 0, w: 100, h: 100 };
		const south = resizeBox(box, 's', 0, 40, true);
		expect(south).toEqual({ x: 0, y: 0, w: 100, h: 140 });
		const north = resizeBox(box, 'n', 0, 30, true);
		expect(north).toEqual({ x: 0, y: 30, w: 100, h: 70 });
	});
});
