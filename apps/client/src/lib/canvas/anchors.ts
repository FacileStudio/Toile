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
