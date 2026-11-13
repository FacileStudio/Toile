import { getStroke } from 'perfect-freehand';
import type { Shape } from '$lib/board/schema';
import type { Point } from '../camera';
import { routePoints, strokeWidth } from '../bounds';
import type { DrawEnv } from './shapes';

const strokes = new WeakMap<Shape, Path2D>();

/** The perfect-freehand outline of flat `[x, y, pressure, ...]` points as a closed path, offset by `(ox, oy)`. */
export function inkPath(points: number[], size: number, ox = 0, oy = 0): Path2D {
	const input: number[][] = [];
	let pressured = false;
	for (let i = 0; i + 2 < points.length; i += 3) {
		input.push([points[i] + ox, points[i + 1] + oy, points[i + 2]]);
		if (points[i + 2] !== 0.5) pressured = true;
	}
	const outline = getStroke(input, {
		size,
		thinning: 0.6,
		smoothing: 0.5,
		streamline: 0.5,
		simulatePressure: !pressured
	});
	const path = new Path2D();
	outline.forEach(([x, y], i) => (i === 0 ? path.moveTo(x, y) : path.lineTo(x, y)));
	path.closePath();
	return path;
}

/** Draws an ink stroke, caching its outline per snapshot. */
export function drawInk(ctx: CanvasRenderingContext2D, shape: Shape, env: DrawEnv): void {
	let path = strokes.get(shape);
	if (!path) strokes.set(shape, (path = inkPath(shape.points ?? [], strokeWidth(shape) * 2, shape.x, shape.y)));
	ctx.fillStyle = shape.style.stroke ?? env.palette.fg;
	ctx.fill(path);
}

function head(ctx: CanvasRenderingContext2D, tip: Point, tail: Point, size: number): void {
	const angle = Math.atan2(tip.y - tail.y, tip.x - tail.x);
	ctx.beginPath();
	ctx.moveTo(tip.x, tip.y);
	ctx.lineTo(tip.x - size * Math.cos(angle - 0.45), tip.y - size * Math.sin(angle - 0.45));
	ctx.lineTo(tip.x - size * Math.cos(angle + 0.45), tip.y - size * Math.sin(angle + 0.45));
	ctx.closePath();
	ctx.fill();
}

/** The corner radius of a connector bound to a shape, in world units. */
export const CONNECTOR_RADIUS = 12;

/** Adds `points` to a new path, each interior corner rounded to at most `radius` and half its shorter segment. */
export function traceRounded(ctx: CanvasRenderingContext2D, points: Point[], radius: number): void {
	ctx.beginPath();
	if (!points.length) return;
	ctx.moveTo(points[0].x, points[0].y);
	for (let i = 1; i < points.length - 1; i += 1) {
		const [p, c, n] = [points[i - 1], points[i], points[i + 1]];
		const r = Math.min(radius, Math.hypot(c.x - p.x, c.y - p.y) / 2, Math.hypot(n.x - c.x, n.y - c.y) / 2);
		ctx.arcTo(c.x, c.y, n.x, n.y, r);
	}
	const last = points[points.length - 1];
	ctx.lineTo(last.x, last.y);
}

/** Strokes a polyline with optional heads, in world space; interior corners round by up to `radius`. */
export function strokePolyline(
	ctx: CanvasRenderingContext2D,
	points: Point[],
	width: number,
	heads: 'none' | 'end' | 'both',
	radius = 0
): void {
	if (points.length < 2) return;
	ctx.lineWidth = width;
	ctx.lineCap = 'round';
	ctx.lineJoin = 'round';
	traceRounded(ctx, points, radius);
	ctx.stroke();
	const last = points[points.length - 1];
	const size = 8 + width * 2;
	if (heads !== 'none') head(ctx, last, points[points.length - 2], size);
	if (heads === 'both') head(ctx, points[0], points[1], size);
}

/** Draws a line or an arrow, bound ends resolved; a connector bound to a shape rounds its elbows. */
export function drawArrow(ctx: CanvasRenderingContext2D, shape: Shape, env: DrawEnv): void {
	const color = shape.style.stroke ?? env.palette.fg;
	ctx.strokeStyle = color;
	ctx.fillStyle = color;
	const heads = shape.type === 'line' ? 'none' : (shape.head ?? 'end');
	const bound = Boolean((shape.from && env.lookup(shape.from.shape)) || (shape.to && env.lookup(shape.to.shape)));
	strokePolyline(ctx, routePoints(shape, env.lookup), strokeWidth(shape), heads, bound ? CONNECTOR_RADIUS : 0);
}
