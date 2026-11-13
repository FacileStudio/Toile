import type { Shape } from '$lib/board/schema';
import { localPoints, routePoints, strokeWidth, type Lookup } from './bounds';
import type { Box, Point } from './camera';
import { center, contains, intersects, rotate, rotatedBox } from './geometry';
import type { Scene } from './scene';

/** Distance from `p` to segment `a`-`b`. */
function segmentDistance(p: Point, a: Point, b: Point): number {
	const dx = b.x - a.x;
	const dy = b.y - a.y;
	const length = dx * dx + dy * dy;
	const t = length ? Math.max(0, Math.min(1, ((p.x - a.x) * dx + (p.y - a.y) * dy) / length)) : 0;
	return Math.hypot(p.x - (a.x + t * dx), p.y - (a.y + t * dy));
}

/** The height of a frame's name label above its box, in world units. */
export const FRAME_LABEL = 24;

function polylineDistance(p: Point, points: Point[]): number {
	let best = Infinity;
	for (let i = 1; i < points.length; i += 1) best = Math.min(best, segmentDistance(p, points[i - 1], points[i]));
	return points.length === 1 ? Math.hypot(p.x - points[0].x, p.y - points[0].y) : best;
}

function insideBox(shape: Shape, local: Point): boolean {
	const nx = (local.x - shape.x) / (shape.w || 1) - 0.5;
	const ny = (local.y - shape.y) / (shape.h || 1) - 0.5;
	if (shape.type === 'ellipse') return nx * nx + ny * ny <= 0.25;
	if (shape.type === 'diamond') return Math.abs(nx) + Math.abs(ny) <= 0.5;
	return Math.abs(nx) <= 0.5 && Math.abs(ny) <= 0.5;
}

function frameHit(shape: Shape, local: Point, tolerance: number): boolean {
	const onLabel =
		local.x >= shape.x &&
		local.x <= shape.x + Math.max(120, shape.w / 2) &&
		local.y >= shape.y - FRAME_LABEL &&
		local.y <= shape.y;
	const outer = {
		x: shape.x - tolerance,
		y: shape.y - tolerance,
		w: shape.w + tolerance * 2,
		h: shape.h + tolerance * 2
	};
	const inner = {
		x: shape.x + tolerance,
		y: shape.y + tolerance,
		w: shape.w - tolerance * 2,
		h: shape.h - tolerance * 2
	};
	const point = { x: local.x, y: local.y, w: 0, h: 0 };
	return onLabel || (contains(outer, point) && !contains(inner, point));
}

/** Whether world point `p` hits `shape`, with `tolerance` world units of slack for thin strokes and edges. */
export function hitShape(shape: Shape, p: Point, tolerance: number, lookup: Lookup): boolean {
	if (shape.type === 'line' || shape.type === 'arrow') {
		return polylineDistance(p, routePoints(shape, lookup)) <= strokeWidth(shape) / 2 + tolerance;
	}
	const local = rotate(p, center(shape), -shape.rot);
	if (shape.type === 'ink') {
		const points = localPoints(shape).map((q) => ({ x: shape.x + q.x, y: shape.y + q.y }));
		return polylineDistance(local, points) <= strokeWidth(shape) / 2 + tolerance;
	}
	if (shape.type === 'frame') return frameHit(shape, local, tolerance);
	return insideBox(shape, local);
}

/** The topmost shape under world point `p`, or null. `skip` filters shapes out (the one being dragged). */
export function topHit(scene: Scene, p: Point, tolerance: number, skip?: (shape: Shape) => boolean): Shape | null {
	const near = scene.within({ x: p.x - tolerance, y: p.y - tolerance, w: tolerance * 2, h: tolerance * 2 });
	for (let i = near.length - 1; i >= 0; i -= 1) {
		const shape = near[i];
		if (!skip?.(shape) && hitShape(shape, p, tolerance, scene.get)) return shape;
	}
	return null;
}

/** The shapes a marquee selects: frames only when wholly inside it, everything else when touched. Locked skip. */
export function marqueeHits(scene: Scene, area: Box): Shape[] {
	return scene.within(area).filter((shape) => {
		if (shape.locked) return false;
		const box = shape.type === 'frame' ? rotatedBox(shape) : (scene.index.box(shape.id) as Box);
		return shape.type === 'frame' ? contains(area, box) : intersects(area, box);
	});
}

/** The topmost frame whose box holds world point `p`, ignoring `skip`. */
export function frameAt(scene: Scene, p: Point, skip: ReadonlySet<string>): Shape | null {
	const near = scene.within({ x: p.x, y: p.y, w: 0, h: 0 });
	for (let i = near.length - 1; i >= 0; i -= 1) {
		const shape = near[i];
		if (shape.type === 'frame' && !skip.has(shape.id) && insideBox(shape, rotate(p, center(shape), -shape.rot)))
			return shape;
	}
	return null;
}
