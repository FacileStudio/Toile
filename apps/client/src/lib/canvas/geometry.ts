import type { Shape } from '$lib/board/schema';
import type { Box, Point } from './camera';

/** Rotates `p` by `angle` radians around `c`. */
export function rotate(p: Point, c: Point, angle: number): Point {
	if (!angle) return p;
	const cos = Math.cos(angle);
	const sin = Math.sin(angle);
	const dx = p.x - c.x;
	const dy = p.y - c.y;
	return { x: c.x + dx * cos - dy * sin, y: c.y + dx * sin + dy * cos };
}

/** The centre of a box. */
export function center(box: Box): Point {
	return { x: box.x + box.w / 2, y: box.y + box.h / 2 };
}

/** The smallest box holding every point. */
export function boxOf(points: Point[]): Box {
	let minX = Infinity;
	let minY = Infinity;
	let maxX = -Infinity;
	let maxY = -Infinity;
	for (const p of points) {
		minX = Math.min(minX, p.x);
		minY = Math.min(minY, p.y);
		maxX = Math.max(maxX, p.x);
		maxY = Math.max(maxY, p.y);
	}
	return points.length ? { x: minX, y: minY, w: maxX - minX, h: maxY - minY } : { x: 0, y: 0, w: 0, h: 0 };
}

/** The box holding every box, or null for none. */
export function union(boxes: Box[]): Box | null {
	if (!boxes.length) return null;
	return boxOf(
		boxes.flatMap((b) => [
			{ x: b.x, y: b.y },
			{ x: b.x + b.w, y: b.y + b.h }
		])
	);
}

/** The four corners of a shape's box in world space, rotation applied. */
export function corners(shape: Pick<Shape, 'x' | 'y' | 'w' | 'h' | 'rot'>): Point[] {
	const c = center(shape);
	const raw = [
		{ x: shape.x, y: shape.y },
		{ x: shape.x + shape.w, y: shape.y },
		{ x: shape.x + shape.w, y: shape.y + shape.h },
		{ x: shape.x, y: shape.y + shape.h }
	];
	return raw.map((p) => rotate(p, c, shape.rot));
}

/** The axis-aligned world box of a box shape, rotation applied. */
export function rotatedBox(shape: Pick<Shape, 'x' | 'y' | 'w' | 'h' | 'rot'>): Box {
	return shape.rot ? boxOf(corners(shape)) : { x: shape.x, y: shape.y, w: shape.w, h: shape.h };
}

/** Whether two boxes overlap. */
export function intersects(a: Box, b: Box): boolean {
	return a.x <= b.x + b.w && b.x <= a.x + a.w && a.y <= b.y + b.h && b.y <= a.y + a.h;
}

/** Whether `inner` lies wholly inside `outer`. */
export function contains(outer: Box, inner: Box): boolean {
	return (
		inner.x >= outer.x &&
		inner.y >= outer.y &&
		inner.x + inner.w <= outer.x + outer.w &&
		inner.y + inner.h <= outer.y + outer.h
	);
}
