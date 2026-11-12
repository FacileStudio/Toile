import { getStroke } from 'perfect-freehand';
import type { Shape } from '$lib/board/schema';
import { cubicCurve, type CubicCurve } from '../anchors';
import type { Point } from '../camera';
import { pathPoints, routePoints, strokeWidth } from '../bounds';
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

/** Strokes a polyline with optional heads, in world space. */
export function strokePolyline(
	ctx: CanvasRenderingContext2D,
	points: Point[],
	width: number,
	heads: 'none' | 'end' | 'both'
): void {
	if (points.length < 2) return;
	ctx.lineWidth = width;
	ctx.lineCap = 'round';
	ctx.lineJoin = 'round';
	ctx.beginPath();
	points.forEach((p, i) => (i === 0 ? ctx.moveTo(p.x, p.y) : ctx.lineTo(p.x, p.y)));
	ctx.stroke();
	const size = 8 + width * 2;
	if (heads !== 'none') head(ctx, points[points.length - 1], points[points.length - 2], size);
	if (heads === 'both') head(ctx, points[0], points[1], size);
}

/** Draws a bezier curve connector with optional heads. */
export function strokeBezier(
	ctx: CanvasRenderingContext2D,
	curve: CubicCurve,
	width: number,
	heads: 'none' | 'end' | 'both'
): void {
	ctx.lineWidth = width;
	ctx.lineCap = 'round';
	ctx.lineJoin = 'round';
	ctx.beginPath();
	ctx.moveTo(curve.a.x, curve.a.y);
	ctx.bezierCurveTo(curve.c1.x, curve.c1.y, curve.c2.x, curve.c2.y, curve.b.x, curve.b.y);
	ctx.stroke();
	const size = 8 + width * 2;
	if (heads !== 'none') head(ctx, curve.b, curve.c2, size);
	if (heads === 'both') head(ctx, curve.a, curve.c1, size);
}

/** Draws a line or an arrow, bound ends resolved. */
export function drawArrow(ctx: CanvasRenderingContext2D, shape: Shape, env: DrawEnv): void {
	const color = shape.style.stroke ?? env.palette.fg;
	ctx.strokeStyle = color;
	ctx.fillStyle = color;
	const heads = shape.type === 'line' ? 'none' : (shape.head ?? 'end');
	const from = shape.from && env.lookup(shape.from.shape);
	const to = shape.to && env.lookup(shape.to.shape);
	if ((from || to) && shape.points?.length === 4) {
		const pts = pathPoints(shape, env.lookup);
		if (pts.length === 2) {
			const bz = cubicCurve(
				{ point: pts[0], shape: from, anchor: shape.from?.anchor },
				{ point: pts[1], shape: to, anchor: shape.to?.anchor }
			);
			strokeBezier(ctx, bz, strokeWidth(shape), heads);
			return;
		}
	}
	strokePolyline(ctx, routePoints(shape, env.lookup), strokeWidth(shape), heads);
}
