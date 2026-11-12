import type { Shape } from '$lib/board/schema';
import { FADE_MS, POP_MS } from '../scene';

/** What the renderer animates this frame: pop-ins, fade-outs and the shapes lifted by a drag. */
export type MotionInput = {
	now: number;
	births: ReadonlyMap<string, number>;
	ghosts: ReadonlyMap<string, { shape: Shape; at: number }>;
	lifted: ReadonlySet<string>;
};

const LIFTABLE: ReadonlySet<Shape['type']> = new Set(['sticky', 'card']);

const liftable = (shape: Shape) => LIFTABLE.has(shape.type) || (shape.type === 'rect' && !!shape.style.fill);

const outBack = (t: number) => 1 + 2.4 * (t - 1) ** 3 + 1.4 * (t - 1) ** 2;
const outCubic = (t: number) => 1 - (1 - t) ** 3;

function around(ctx: CanvasRenderingContext2D, shape: Shape, scale: number, tilt = 0): void {
	const cx = shape.x + shape.w / 2;
	const cy = shape.y + shape.h / 2;
	ctx.translate(cx, cy);
	if (tilt) ctx.rotate(tilt);
	ctx.scale(scale, scale);
	ctx.translate(-cx, -cy);
}

function lift(ctx: CanvasRenderingContext2D, shape: Shape): void {
	const alpha = ctx.globalAlpha;
	around(ctx, shape, 1.03, shape.type === 'sticky' ? -0.02 : 0);
	ctx.shadowColor = 'black';
	ctx.shadowBlur = 28;
	ctx.shadowOffsetY = 12;
	ctx.globalAlpha = alpha * 0.22;
	ctx.fillStyle = 'black';
	ctx.save();
	around(ctx, shape, 1, shape.rot);
	ctx.beginPath();
	ctx.roundRect(shape.x, shape.y, shape.w, shape.h, shape.type === 'sticky' ? 4 : 8);
	ctx.fill();
	ctx.restore();
	ctx.shadowColor = 'transparent';
	ctx.globalAlpha = alpha;
}

/**
 * Draws `shape` through `draw` inside its animation, if it has one: a springy scale-up from 82% as it appears, a
 * lift with a soft shadow (and a slight tilt for stickies) while a drag holds it.
 */
export function drawMoving(
	ctx: CanvasRenderingContext2D,
	shape: Shape,
	motion: MotionInput,
	draw: () => void
): void {
	const born = motion.births.get(shape.id);
	const lifted = motion.lifted.has(shape.id) && liftable(shape);
	if (born === undefined && !lifted) return draw();
	ctx.save();
	if (born !== undefined) {
		const t = Math.min(1, (motion.now - born) / POP_MS);
		ctx.globalAlpha = Math.min(1, t * 2.5);
		around(ctx, shape, 0.82 + 0.18 * outBack(t));
	}
	if (lifted) lift(ctx, shape);
	draw();
	ctx.restore();
}

/** Draws the shapes that just left, shrinking to 78% as they fade. */
export function drawGhosts(ctx: CanvasRenderingContext2D, motion: MotionInput, draw: (shape: Shape) => void): void {
	for (const { shape, at } of motion.ghosts.values()) {
		const t = Math.min(1, (motion.now - at) / FADE_MS);
		ctx.save();
		ctx.globalAlpha = 1 - outCubic(t);
		around(ctx, shape, 1 - 0.22 * outCubic(t));
		draw(shape);
		ctx.restore();
	}
}
