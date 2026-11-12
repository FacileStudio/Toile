import type { Shape } from '$lib/board/schema';
import type { Point } from './camera';
import { center, rotate } from './geometry';

/** Distance from `p` to segment `a`-`b`. */
export function segmentDistance(p: Point, a: Point, b: Point): number {
	const dx = b.x - a.x;
	const dy = b.y - a.y;
	const length = dx * dx + dy * dy;
	const t = length ? Math.max(0, Math.min(1, ((p.x - a.x) * dx + (p.y - a.y) * dy) / length)) : 0;
	return Math.hypot(p.x - (a.x + t * dx), p.y - (a.y + t * dy));
}

/** The world point of a fractional anchor on a shape's box, rotation applied. */
export function anchorPoint(shape: Pick<Shape, 'x' | 'y' | 'w' | 'h' | 'rot'>, anchor: [number, number]): Point {
	const p = { x: shape.x + anchor[0] * shape.w, y: shape.y + anchor[1] * shape.h };
	return rotate(p, center(shape), shape.rot);
}

/** The fractional anchor of world point `p` on a shape, clamped to its box. */
export function anchorOf(shape: Pick<Shape, 'x' | 'y' | 'w' | 'h' | 'rot'>, p: Point): [number, number] {
	const local = rotate(p, center(shape), -shape.rot);
	const fx = shape.w ? (local.x - shape.x) / shape.w : 0.5;
	const fy = shape.h ? (local.y - shape.y) / shape.h : 0.5;
	return [Math.max(0, Math.min(1, fx)), Math.max(0, Math.min(1, fy))];
}

/** One end of a connector: its world point and, when bound, the shape and anchor it sits on. */
export type End = { point: Point; shape?: Pick<Shape, 'x' | 'y' | 'w' | 'h' | 'rot'>; anchor?: [number, number] };

const CURVE_SAMPLES = 64;

function axis(dx: number, dy: number): Point {
	if (Math.abs(dx) >= Math.abs(dy)) return { x: Math.sign(dx) || 1, y: 0 };
	return { x: 0, y: Math.sign(dy) || 1 };
}

/** The unit direction a connector leaves `end` in: out of the side its anchor sits on, or toward `other`. */
export function exitDirection(end: End, other: Point): Point {
	const toward = axis(other.x - end.point.x, other.y - end.point.y);
	if (!end.shape || !end.anchor) return toward;
	const dx = end.anchor[0] - 0.5;
	const dy = end.anchor[1] - 0.5;
	if (Math.abs(dx) < 0.05 && Math.abs(dy) < 0.05) return toward;
	const fromX = (0.5 - Math.abs(dx)) * Math.max(end.shape.w, 1);
	const fromY = (0.5 - Math.abs(dy)) * Math.max(end.shape.h, 1);
	const side = fromX <= fromY ? { x: Math.sign(dx) || 1, y: 0 } : { x: 0, y: Math.sign(dy) || 1 };
	return rotate(side, { x: 0, y: 0 }, end.shape.rot);
}

/** A connector's cubic curve from `a` to `b`, sampled as a polyline; each end leaves perpendicular to its shape. */
export function curve(a: End, b: End): Point[] {
	const distance = Math.hypot(b.point.x - a.point.x, b.point.y - a.point.y);
	const reach = Math.max(distance * 0.45, 32);
	const da = exitDirection(a, b.point);
	const db = exitDirection(b, a.point);
	const c1 = { x: a.point.x + da.x * reach, y: a.point.y + da.y * reach };
	const c2 = { x: b.point.x + db.x * reach, y: b.point.y + db.y * reach };
	const out: Point[] = [];
	for (let i = 0; i <= CURVE_SAMPLES; i += 1) {
		const t = i / CURVE_SAMPLES;
		const u = 1 - t;
		const k0 = u * u * u;
		const k1 = 3 * u * u * t;
		const k2 = 3 * u * t * t;
		const k3 = t * t * t;
		out.push({
			x: k0 * a.point.x + k1 * c1.x + k2 * c2.x + k3 * b.point.x,
			y: k0 * a.point.y + k1 * c1.y + k2 * c2.y + k3 * b.point.y
		});
	}
	return out;
}
