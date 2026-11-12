import type { Shape } from '$lib/board/schema';
import type { Lookup } from '../bounds';
import { strokeWidth } from '../bounds';
import { FRAME_LABEL } from '../hit';
import { drawCard } from './cards';
import { inkOn, STICKY_FILL, type Palette } from './palette';
import { drawArrow, drawInk } from './paths';
import { drawText, fontSize } from './text';

/** What drawing a shape needs besides the shape. */
export type DrawEnv = {
	palette: Palette;
	lookup: Lookup;
	zoom: number;
	editing: string | null;
	image: (asset: string) => CanvasImageSource | null;
};

function outline(ctx: CanvasRenderingContext2D, shape: Shape, radius: number): void {
	const { x, y, w, h } = shape;
	ctx.beginPath();
	if (shape.type === 'ellipse') ctx.ellipse(x + w / 2, y + h / 2, w / 2, h / 2, 0, 0, Math.PI * 2);
	else if (shape.type === 'diamond') {
		ctx.moveTo(x + w / 2, y);
		ctx.lineTo(x + w, y + h / 2);
		ctx.lineTo(x + w / 2, y + h);
		ctx.lineTo(x, y + h / 2);
		ctx.closePath();
	} else ctx.roundRect(x, y, w, h, radius);
}

function drawSticky(ctx: CanvasRenderingContext2D, shape: Shape, env: DrawEnv): void {
	const fill = shape.style.fill ?? STICKY_FILL;
	ctx.fillStyle = fill;
	outline(ctx, shape, 4);
	ctx.fill();
	if (env.editing !== shape.id && fontSize(shape) * env.zoom >= 3) drawText(ctx, shape, inkOn(fill), env.palette.font);
}

function drawBox(ctx: CanvasRenderingContext2D, shape: Shape, env: DrawEnv): void {
	outline(ctx, shape, 8);
	if (shape.style.fill) {
		ctx.fillStyle = shape.style.fill;
		ctx.fill();
	}
	ctx.strokeStyle = shape.style.stroke ?? env.palette.fg;
	ctx.lineWidth = strokeWidth(shape);
	ctx.stroke();
	const color = shape.style.fill ? inkOn(shape.style.fill) : env.palette.fg;
	if (env.editing !== shape.id && fontSize(shape) * env.zoom >= 3) drawText(ctx, shape, color, env.palette.font);
}

function drawFrame(ctx: CanvasRenderingContext2D, shape: Shape, env: DrawEnv): void {
	ctx.fillStyle = shape.style.fill ?? env.palette.component;
	ctx.beginPath();
	ctx.roundRect(shape.x, shape.y, shape.w, shape.h, 12);
	ctx.fill();
	ctx.fillStyle = env.palette.muted;
	ctx.font = `500 ${Math.max(14, 14 / env.zoom)}px ${env.palette.font}`;
	ctx.textAlign = 'left';
	ctx.textBaseline = 'bottom';
	ctx.fillText(shape.name || 'Cadre', shape.x + 2, shape.y - Math.min(FRAME_LABEL, 6 / env.zoom));
}

function drawMedia(ctx: CanvasRenderingContext2D, shape: Shape, env: DrawEnv): void {
	if (shape.type === 'card') return drawCard(ctx, shape, env);
	const image = shape.asset ? env.image(shape.asset) : null;
	ctx.save();
	ctx.beginPath();
	ctx.roundRect(shape.x, shape.y, shape.w, shape.h, 8);
	ctx.fillStyle = env.palette.surface;
	ctx.fill();
	ctx.clip();
	if (image) {
		ctx.imageSmoothingEnabled = true;
		ctx.drawImage(image, shape.x, shape.y, shape.w, shape.h);
	} else if (14 * env.zoom >= 3) {
		ctx.fillStyle = env.palette.muted;
		ctx.font = `500 14px ${env.palette.font}`;
		ctx.textAlign = 'center';
		ctx.textBaseline = 'middle';
		ctx.fillText('Image', shape.x + shape.w / 2, shape.y + shape.h / 2);
	}
	ctx.restore();
}

function drawPlainText(ctx: CanvasRenderingContext2D, shape: Shape, env: DrawEnv): void {
	if (env.editing === shape.id || fontSize(shape) * env.zoom < 3) return;
	drawText(ctx, shape, shape.style.stroke ?? env.palette.fg, env.palette.font);
}

const DRAW: Record<Shape['type'], (ctx: CanvasRenderingContext2D, shape: Shape, env: DrawEnv) => void> = {
	sticky: drawSticky,
	text: drawPlainText,
	rect: drawBox,
	ellipse: drawBox,
	diamond: drawBox,
	line: drawArrow,
	arrow: drawArrow,
	ink: drawInk,
	image: drawMedia,
	card: drawMedia,
	frame: drawFrame
};

/** Draws one shape in world space, rotation applied around its box centre. */
export function drawShape(ctx: CanvasRenderingContext2D, shape: Shape, env: DrawEnv): void {
	const rotated = shape.rot !== 0 && shape.type !== 'line' && shape.type !== 'arrow';
	if (rotated) {
		ctx.save();
		ctx.translate(shape.x + shape.w / 2, shape.y + shape.h / 2);
		ctx.rotate(shape.rot);
		ctx.translate(-(shape.x + shape.w / 2), -(shape.y + shape.h / 2));
	}
	DRAW[shape.type](ctx, shape, env);
	if (rotated) ctx.restore();
}
