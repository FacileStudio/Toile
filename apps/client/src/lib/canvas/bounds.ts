import { PATH_TYPES, type Shape } from '$lib/board/schema';
import { anchorPoint } from './anchors';
import type { Box, Point } from './camera';
import { boxOf, center, rotate, rotatedBox } from './geometry';

/** Looks a shape up by id. */
export type Lookup = (id: string) => Shape | undefined;

/** The default stroke width of a shape, in world units. */
export function strokeWidth(shape: Shape): number {
	return shape.style.width ?? (shape.type === 'ink' ? 4 : 2);
}

/** The local points of a path shape, as `[x, y]` pairs; ink drops its pressure column. */
export function localPoints(shape: Shape): Point[] {
	const flat = shape.points ?? [];
	const stride = shape.type === 'ink' ? 3 : 2;
	const out: Point[] = [];
	for (let i = 0; i + 1 < flat.length; i += stride) out.push({ x: flat[i], y: flat[i + 1] });
	return out;
}

/** The world points of a line or arrow, bound ends resolved to their shapes' anchors. */
export function pathPoints(shape: Shape, lookup: Lookup): Point[] {
	const c = center(shape);
	const points = localPoints(shape).map((p) => rotate({ x: shape.x + p.x, y: shape.y + p.y }, c, shape.rot));
	if (points.length < 2) return points;
	const from = shape.from && lookup(shape.from.shape);
	const to = shape.to && lookup(shape.to.shape);
	if (from && shape.from) points[0] = anchorPoint(from, shape.from.anchor);
	if (to && shape.to) points[points.length - 1] = anchorPoint(to, shape.to.anchor);
	return points;
}

/** The world box a shape covers, padded by its stroke so culling never clips an edge. */
export function shapeBounds(shape: Shape, lookup: Lookup): Box {
	const pad = strokeWidth(shape) / 2 + (shape.type === 'arrow' ? 12 : 1);
	const box = shape.type === 'line' || shape.type === 'arrow' ? boxOf(pathPoints(shape, lookup)) : rotatedBox(shape);
	const label = shape.type === 'frame' ? 28 : 0;
	return { x: box.x - pad, y: box.y - pad - label, w: box.w + pad * 2, h: box.h + pad * 2 + label };
}

/** The local box of points, for ink and paths: `[minX, minY, w, h]` relative to the shape origin. */
export function pointsBox(points: number[], stride: number): Box {
	const pairs: Point[] = [];
	for (let i = 0; i + 1 < points.length; i += stride) pairs.push({ x: points[i], y: points[i + 1] });
	return boxOf(pairs);
}

/** Whether a shape is drawn from points rather than a box. */
export const isPath = (shape: Shape) => PATH_TYPES.has(shape.type);
