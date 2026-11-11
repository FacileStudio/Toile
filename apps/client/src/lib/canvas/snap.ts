import type { Box } from './camera';

/** A snap result: the correction to apply and the guide lines to draw, in world units. */
export type Snap = { dx: number; dy: number; guides: { x: number[]; y: number[] } };

const lines = (box: Box, axis: 'x' | 'y') =>
	axis === 'x' ? [box.x, box.x + box.w / 2, box.x + box.w] : [box.y, box.y + box.h / 2, box.y + box.h];

function snapAxis(moving: Box, others: Box[], axis: 'x' | 'y', threshold: number): [number, number[]] {
	let best = threshold;
	let delta = 0;
	let guides: number[] = [];
	const mine = lines(moving, axis);
	const pairs = others.flatMap((other) => lines(other, axis).flatMap((target) => mine.map((line) => [target, line])));
	for (const [target, line] of pairs) {
		const distance = Math.abs(target - line);
		if (distance < best - 1e-9) {
			best = distance;
			delta = target - line;
			guides = [target];
		} else if (Math.abs(distance - best) < 1e-9 && Math.abs(target - line - delta) < 1e-9) guides.push(target);
	}
	return [delta, guides];
}

/**
 * Snaps a moving box's edges and centre to the edges and centres of `others`, per axis, when within `threshold`
 * world units. The closest line wins; equal candidates all show as guides.
 */
export function snapBox(moving: Box, others: Box[], threshold: number): Snap {
	const [dx, gx] = snapAxis(moving, others, 'x', threshold);
	const [dy, gy] = snapAxis(moving, others, 'y', threshold);
	return { dx, dy, guides: { x: [...new Set(gx)], y: [...new Set(gy)] } };
}
