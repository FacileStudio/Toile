import { describe, expect, it } from 'vitest';
import { curve } from './anchors';

describe('connector curve', () => {
	it('starts and ends on its points and leaves each shape out of the side it is anchored on', () => {
		const left = { x: 0, y: 0, w: 100, h: 100, rot: 0 };
		const right = { x: 300, y: 200, w: 100, h: 100, rot: 0 };
		const points = curve(
			{ point: { x: 100, y: 50 }, shape: left, anchor: [1, 0.5] },
			{ point: { x: 300, y: 250 }, shape: right, anchor: [0, 0.5] }
		);
		expect(points[0]).toEqual({ x: 100, y: 50 });
		expect(points.at(-1)).toEqual({ x: 300, y: 250 });
		expect(points[1].x).toBeGreaterThan(100);
		expect(Math.abs(points[1].y - 50)).toBeLessThan(1);
		expect(points.at(-2)!.x).toBeLessThan(300);
		expect(Math.abs(points.at(-2)!.y - 250)).toBeLessThan(1);
	});

	it('leaves a top anchor upward', () => {
		const box = { x: 0, y: 0, w: 100, h: 100, rot: 0 };
		const points = curve({ point: { x: 50, y: 0 }, shape: box, anchor: [0.5, 0] }, { point: { x: 400, y: 400 } });
		expect(points[1].y).toBeLessThan(0);
	});

	it('leaves a wide box out of the edge its anchor is nearest, not the far axis', () => {
		const wide = { x: 0, y: 0, w: 400, h: 100, rot: 0 };
		const points = curve({ point: { x: 340, y: 0 }, shape: wide, anchor: [0.85, 0] }, { point: { x: 600, y: 400 } });
		expect(points[1].y).toBeLessThan(0);
		expect(Math.abs(points[1].x - 340)).toBeLessThan(1);
	});
});
