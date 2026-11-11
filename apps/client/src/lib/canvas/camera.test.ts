import { describe, expect, it } from 'vitest';
import { fitBox, panBy, pinch, toScreen, toWorld, viewport, zoomAt, ZOOM } from './camera';
import { wheelFactor } from './input';

describe('camera', () => {
	it('round-trips screen and world points', () => {
		const camera = { x: 120, y: -40, z: 2.5 };
		const p = { x: 33, y: 77 };
		const back = toScreen(camera, toWorld(camera, p));
		expect(back.x).toBeCloseTo(p.x);
		expect(back.y).toBeCloseTo(p.y);
	});

	it('zooms to the cursor: the world point under it stays under it', () => {
		const camera = { x: 10, y: 20, z: 1 };
		const cursor = { x: 300, y: 200 };
		const before = toWorld(camera, cursor);
		const after = toWorld(zoomAt(camera, cursor, 3), cursor);
		expect(after.x).toBeCloseTo(before.x);
		expect(after.y).toBeCloseTo(before.y);
	});

	it('clamps the zoom range', () => {
		expect(zoomAt({ x: 0, y: 0, z: 1 }, { x: 0, y: 0 }, 1000).z).toBe(ZOOM.max);
		expect(zoomAt({ x: 0, y: 0, z: 1 }, { x: 0, y: 0 }, 0.0001).z).toBe(ZOOM.min);
	});

	it('pans and reports the visible world box', () => {
		const camera = panBy({ x: 0, y: 0, z: 2 }, 100, 50);
		expect(viewport(camera, 800, 600)).toEqual({ x: -50, y: -25, w: 400, h: 300 });
	});

	it('fits a box centred with margin, capped at the max zoom', () => {
		const camera = fitBox({ x: 0, y: 0, w: 1000, h: 500 }, 1100, 600, 50);
		expect(camera.z).toBeCloseTo(1);
		const centre = toScreen(camera, { x: 500, y: 250 });
		expect(centre).toEqual({ x: 550, y: 300 });
	});

	it('pinches: spreading two fingers zooms around their midpoint, moving them pans', () => {
		const camera = { x: 0, y: 0, z: 1 };
		const spread = pinch(
			camera,
			[
				{ x: 90, y: 100 },
				{ x: 110, y: 100 }
			],
			[
				{ x: 80, y: 100 },
				{ x: 120, y: 100 }
			]
		);
		expect(spread.z).toBeCloseTo(2);
		expect(toWorld(spread, { x: 100, y: 100 }).x).toBeCloseTo(100);
		const moved = pinch(
			camera,
			[
				{ x: 0, y: 0 },
				{ x: 10, y: 0 }
			],
			[
				{ x: 5, y: 7 },
				{ x: 15, y: 7 }
			]
		);
		expect(moved).toEqual({ x: 5, y: 7, z: 1 });
	});

	it('zooms in on a negative wheel delta, finer for a mouse than a pinch', () => {
		expect(wheelFactor(-100, 0, false)).toBeGreaterThan(1);
		expect(wheelFactor(100, 0, true)).toBeLessThan(wheelFactor(100, 0, false));
		expect(wheelFactor(3, 1, false)).toBeCloseTo(wheelFactor(48, 0, false));
	});
});
