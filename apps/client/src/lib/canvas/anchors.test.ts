import { describe, expect, it } from 'vitest';
import { elbow, type End } from './anchors';
import type { Point } from './camera';

type Box = { x: number; y: number; w: number; h: number; rot: number };

const axisAligned = (points: Point[]) => points.slice(1).every((q, i) => q.x === points[i].x || q.y === points[i].y);

const crosses = (points: Point[], b: Box) =>
	points.slice(1).some((q, i) => {
		const p = points[i];
		return (
			Math.min(p.x, q.x) < b.x + b.w &&
			Math.max(p.x, q.x) > b.x &&
			Math.min(p.y, q.y) < b.y + b.h &&
			Math.max(p.y, q.y) > b.y
		);
	});

const at = (shape: Box, anchor: [number, number]): End => ({
	point: { x: shape.x + anchor[0] * shape.w, y: shape.y + anchor[1] * shape.h },
	shape,
	anchor
});

describe('connector elbow', () => {
	it('routes shapes side by side as a Z of four points', () => {
		const left = { x: 0, y: 0, w: 100, h: 100, rot: 0 };
		const right = { x: 300, y: 200, w: 100, h: 100, rot: 0 };
		expect(elbow(at(left, [1, 0.5]), at(right, [0, 0.5]))).toEqual([
			{ x: 100, y: 50 },
			{ x: 200, y: 50 },
			{ x: 200, y: 250 },
			{ x: 300, y: 250 }
		]);
	});

	it('draws facing ends on one line as a straight segment', () => {
		const left = { x: 0, y: 0, w: 100, h: 100, rot: 0 };
		const right = { x: 300, y: 0, w: 100, h: 100, rot: 0 };
		expect(elbow(at(left, [1, 0.5]), at(right, [0, 0.5]))).toEqual([
			{ x: 100, y: 50 },
			{ x: 300, y: 50 }
		]);
	});

	it('turns perpendicular exits into an L', () => {
		const left = { x: 0, y: 0, w: 100, h: 100, rot: 0 };
		const below = { x: 300, y: 300, w: 100, h: 100, rot: 0 };
		expect(elbow(at(left, [1, 0.5]), at(below, [0.5, 0]))).toEqual([
			{ x: 100, y: 50 },
			{ x: 350, y: 50 },
			{ x: 350, y: 300 }
		]);
	});

	it('wraps a U around both shapes when both ends exit the same way', () => {
		const cases: [Box, Box][] = [
			[
				{ x: 0, y: 0, w: 100, h: 100, rot: 0 },
				{ x: 0, y: 300, w: 100, h: 100, rot: 0 }
			],
			[
				{ x: 0, y: 0, w: 100, h: 100, rot: 0 },
				{ x: 300, y: 0, w: 100, h: 100, rot: 0 }
			]
		];
		for (const [a, b] of cases) {
			const points = elbow(at(a, [1, 0.5]), at(b, [1, 0.5]));
			expect(axisAligned(points)).toBe(true);
			expect(points[1].x).toBeGreaterThan(100);
			expect(crosses(points, a)).toBe(false);
			expect(crosses(points, b)).toBe(false);
		}
	});

	it('leaves a top anchor upward and routes a free end toward it', () => {
		const box = { x: 0, y: 0, w: 100, h: 100, rot: 0 };
		const points = elbow(at(box, [0.5, 0]), { point: { x: 400, y: 400 } });
		expect(points[1]).toEqual({ x: 50, y: -16 });
		expect(points.at(-1)).toEqual({ x: 400, y: 400 });
		expect(axisAligned(points)).toBe(true);
		expect(crosses(points, box)).toBe(false);
	});

	it('keeps every segment axis-aligned and ends on its points', () => {
		const box = { x: 0, y: 0, w: 120, h: 80, rot: 0 };
		const anchors: [number, number][] = [
			[0, 0.5],
			[1, 0.5],
			[0.5, 0],
			[0.5, 1]
		];
		const offsets = [
			[300, 0],
			[0, 300],
			[-250, 180],
			[60, 40]
		];
		const pairs = anchors.flatMap((ax) =>
			anchors.flatMap((bx) => offsets.map(([x, y]) => [at(box, ax), at({ ...box, x, y }, bx)]))
		);
		for (const [a, b] of pairs) {
			const points = elbow(a, b);
			expect(axisAligned(points)).toBe(true);
			expect(points[0]).toEqual(a.point);
			expect(points.at(-1)).toEqual(b.point);
		}
	});
});

describe('elbow from centre anchors', () => {
	const sticky = (x: number, y: number) => ({ x, y, w: 200, h: 200, rot: 0 });
	const at = (s: ReturnType<typeof sticky>) => ({ point: { x: s.x + 100, y: s.y + 100 }, shape: s, anchor: [0.5, 0.5] as [number, number] });

	it('leaves side by side stickies from their facing edges', () => {
		const route = elbow(at(sticky(0, 0)), at(sticky(400, 0)));
		expect(route[0]).toEqual({ x: 200, y: 100 });
		expect(route[route.length - 1]).toEqual({ x: 400, y: 100 });
	});

	it('joins stacked stickies edge to edge', () => {
		expect(elbow(at(sticky(0, 0)), at(sticky(0, 300)))).toEqual([
			{ x: 100, y: 200 },
			{ x: 100, y: 300 }
		]);
	});
});
