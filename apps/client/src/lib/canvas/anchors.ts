import type { Shape } from '$lib/board/schema';
import type { Box, Point } from './camera';
import { boxOf, center, rotate, rotatedBox } from './geometry';

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

/** One end of a connector: its world point and, when bound, the shape and anchor it sits on. */
export type End = { point: Point; shape?: Pick<Shape, 'x' | 'y' | 'w' | 'h' | 'rot'>; anchor?: [number, number] };

const STUB = 16;
const BEND = 24;
const BLOCKED = 1e6;

function axis(dx: number, dy: number): Point {
	if (Math.abs(dx) >= Math.abs(dy)) return { x: Math.sign(dx) || 1, y: 0 };
	return { x: 0, y: Math.sign(dy) || 1 };
}

/** The unit direction a connector leaves `end` in: out of the side its anchor sits on, or toward `other`. */
export function exitDirection(end: End, other: Point): Point {
	const toward = axis(other.x - end.point.x, other.y - end.point.y);
	if (!end.shape || !end.anchor) return toward;
	const dx = end.anchor[0] - 0.5;
	const dy = end.anchor[1] - 0.5;
	if (Math.abs(dx) < 0.05 && Math.abs(dy) < 0.05) return toward;
	const fromX = (0.5 - Math.abs(dx)) * Math.max(end.shape.w, 1);
	const fromY = (0.5 - Math.abs(dy)) * Math.max(end.shape.h, 1);
	const side = fromX <= fromY ? { x: Math.sign(dx) || 1, y: 0 } : { x: 0, y: Math.sign(dy) || 1 };
	return rotate(side, { x: 0, y: 0 }, end.shape.rot);
}

function simplify(points: Point[]): Point[] {
	const out: Point[] = [points[0]];
	for (const p of points.slice(1)) {
		const last = out[out.length - 1];
		if (Math.abs(last.x - p.x) < 1e-6 && Math.abs(last.y - p.y) < 1e-6) continue;
		const prev = out[out.length - 2] ?? last;
		const [ux, uy, vx, vy] = [last.x - prev.x, last.y - prev.y, p.x - last.x, p.y - last.y];
		if (Math.abs(ux * vy - uy * vx) < 1e-6 && ux * vx + uy * vy > 0) out.pop();
		out.push(p);
	}
	return out;
}

function cost(route: Point[], boxes: (Box | undefined)[]): number {
	let total = (route.length - 2) * BEND;
	for (let i = 1; i < route.length; i += 1) {
		const [p, q, before = p] = [route[i - 1], route[i], route[i - 2]];
		total += Math.abs(q.x - p.x) + Math.abs(q.y - p.y);
		if ((q.x - p.x) * (p.x - before.x) + (q.y - p.y) * (p.y - before.y) < 0) total += BLOCKED;
		const own = i === 1 ? boxes[0] : i === route.length - 1 ? boxes[1] : undefined;
		const span = boxOf([p, q]);
		for (const b of boxes) {
			if (!b || b === own) continue;
			const xs = span.x < b.x + b.w - 0.5 && span.x + span.w > b.x + 0.5;
			if (xs && span.y < b.y + b.h - 0.5 && span.y + span.h > b.y + 0.5) total += BLOCKED;
		}
	}
	return total;
}

/** A connector's orthogonal route from `a` to `b`: a stub out of each end, then the cheapest path of one or two bends that avoids both shapes. */
export function elbow(a: End, b: End): Point[] {
	const da = exitDirection(a, b.point);
	const db = exitDirection(b, a.point);
	const ea = axis(da.x, da.y);
	const eb = axis(db.x, db.y);
	const sa = { x: a.point.x + ea.x * STUB, y: a.point.y + ea.y * STUB };
	const sb = { x: b.point.x + eb.x * STUB, y: b.point.y + eb.y * STUB };
	const boxes = [a.shape && rotatedBox(a.shape), b.shape && rotatedBox(b.shape)];
	const hull = boxOf([sa, sb, ...boxes.flatMap((box) => (box ? [box, { x: box.x + box.w, y: box.y + box.h }] : []))]);
	const xs = [(sa.x + sb.x) / 2, sa.x, sb.x, hull.x - STUB, hull.x + hull.w + STUB];
	const ys = [(sa.y + sb.y) / 2, sa.y, sb.y, hull.y - STUB, hull.y + hull.h + STUB];
	const routes = [
		...xs.map((x) => [a.point, sa, { x, y: sa.y }, { x, y: sb.y }, sb, b.point]),
		...ys.map((y) => [a.point, sa, { x: sa.x, y }, { x: sb.x, y }, sb, b.point])
	].map(simplify);
	const scores = routes.map((route) => cost(route, boxes));
	return routes[scores.indexOf(Math.min(...scores))];
}
