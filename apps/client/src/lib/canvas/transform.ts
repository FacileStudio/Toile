import { PATH_TYPES, type Shape, type ShapeFields } from '$lib/board/schema';
import type { Box, Point } from './camera';

/** A resize handle: the edges it moves. */
export type Handle = 'n' | 's' | 'e' | 'w' | 'ne' | 'nw' | 'se' | 'sw';

/** Every resize handle, corners first. */
export const HANDLES: Handle[] = ['nw', 'ne', 'se', 'sw', 'n', 'e', 's', 'w'];

/** The four corner resize handles. */
export const CORNERS: Handle[] = ['nw', 'ne', 'se', 'sw'];

/** The four edge resize handles. */
export const EDGES: Handle[] = ['n', 'e', 's', 'w'];

const MIN = 8;

/**
 * The box `box` becomes when `handle` is dragged by `(dx, dy)` in the box's own frame. The opposite edge stays put
 * and the size never drops under 8 units; `keepAspect` locks the ratio on corners.
 */
export function resizeBox(box: Box, handle: Handle, dx: number, dy: number, keepAspect: boolean): Box {
	let left = box.x;
	let top = box.y;
	let right = box.x + box.w;
	let bottom = box.y + box.h;
	if (handle.includes('w')) left += dx;
	if (handle.includes('e')) right += dx;
	if (handle.includes('n')) top += dy;
	if (handle.includes('s')) bottom += dy;
	let w = Math.max(MIN, right - left);
	let h = Math.max(MIN, bottom - top);
	if (keepAspect && handle.length === 2 && box.w > 0 && box.h > 0) {
		const scale = Math.max(w / box.w, h / box.h);
		w = box.w * scale;
		h = box.h * scale;
	}
	const x = handle.includes('w') ? box.x + box.w - w : box.x;
	const y = handle.includes('n') ? box.y + box.h - h : box.y;
	return { x, y, w, h };
}

/** The fields that map `shape` from group box `from` into group box `to`: position, size and scaled points. */
export function scaleInto(shape: Shape, from: Box, to: Box): ShapeFields {
	const sx = from.w ? to.w / from.w : 1;
	const sy = from.h ? to.h / from.h : 1;
	const fields: ShapeFields = {
		x: to.x + (shape.x - from.x) * sx,
		y: to.y + (shape.y - from.y) * sy,
		w: Math.max(1, shape.w * sx),
		h: Math.max(1, shape.h * sy)
	};
	if (PATH_TYPES.has(shape.type) && shape.points) {
		const stride = shape.type === 'ink' ? 3 : 2;
		fields.points = shape.points.map((value, i) =>
			i % stride === 0 ? value * sx : i % stride === 1 ? value * sy : value
		);
	}
	return fields;
}
