import type { Shape } from '$lib/board/schema';
import { anchorPoint } from '../anchors';
import { routePoints } from '../bounds';
import type { Box, Camera, Point } from '../camera';
import { toScreen } from '../camera';
import { corners } from '../geometry';
import type { Scene } from '../scene';
import { fitCanvas } from './renderer';
import { CONNECTOR_RADIUS, inkPath, strokePolyline } from './paths';
import type { Palette } from './palette';

/** A peer as the overlay draws them. */
export type PeerView = { id: number; name: string; color: string; cursor: Point | null; selection: string[] };

/** A gesture in progress that is not in the document yet. */
export type Draft =
	| { kind: 'ink'; points: number[]; width: number; color: string }
	| { kind: 'connector'; points: Point[]; color: string; head: boolean }
	| { kind: 'eraser'; points: Point[] };

/** Everything the overlay canvas shows over the scene. */
export type OverlayState = {
	selection: readonly string[];
	hover: string | null;
	marquee: Box | null;
	guides: { x: number[]; y: number[] };
	draft: Draft | null;
	target: { shape: string; anchor: [number, number] } | null;
	peers: PeerView[];
};

function outlineShape(ctx: CanvasRenderingContext2D, shape: Shape, scene: Scene, camera: Camera): void {
	const points = shape.type === 'line' || shape.type === 'arrow' ? routePoints(shape, scene.get) : corners(shape);
	ctx.beginPath();
	points.map((p) => toScreen(camera, p)).forEach((p, i) => (i === 0 ? ctx.moveTo(p.x, p.y) : ctx.lineTo(p.x, p.y)));
	if (shape.type !== 'line' && shape.type !== 'arrow') ctx.closePath();
	ctx.stroke();
}

function drawCursor(ctx: CanvasRenderingContext2D, peer: PeerView, camera: Camera, palette: Palette): void {
	if (!peer.cursor) return;
	const p = toScreen(camera, peer.cursor);
	ctx.fillStyle = peer.color;
	ctx.beginPath();
	ctx.moveTo(p.x, p.y);
	ctx.lineTo(p.x + 4, p.y + 16);
	ctx.lineTo(p.x + 8, p.y + 10);
	ctx.lineTo(p.x + 15, p.y + 9);
	ctx.closePath();
	ctx.fill();
	ctx.font = `500 12px ${palette.font}`;
	const width = ctx.measureText(peer.name).width + 12;
	ctx.beginPath();
	ctx.roundRect(p.x + 12, p.y + 16, width, 20, 10);
	ctx.fill();
	ctx.fillStyle = 'black';
	ctx.textBaseline = 'middle';
	ctx.textAlign = 'left';
	ctx.fillText(peer.name, p.x + 18, p.y + 26);
}

function drawGuides(ctx: CanvasRenderingContext2D, state: OverlayState, camera: Camera, size: Point): void {
	ctx.beginPath();
	for (const x of state.guides.x) {
		const sx = toScreen(camera, { x, y: 0 }).x;
		ctx.moveTo(sx, 0);
		ctx.lineTo(sx, size.y);
	}
	for (const y of state.guides.y) {
		const sy = toScreen(camera, { x: 0, y }).y;
		ctx.moveTo(0, sy);
		ctx.lineTo(size.x, sy);
	}
	ctx.stroke();
}

function drawDraft(ctx: CanvasRenderingContext2D, draft: Draft, camera: Camera, palette: Palette): void {
	ctx.save();
	ctx.setTransform(
		ctx
			.getTransform()
			.scale(camera.z)
			.translateSelf(camera.x / camera.z, camera.y / camera.z)
	);
	if (draft.kind === 'ink') {
		ctx.fillStyle = draft.color;
		ctx.fill(inkPath(draft.points, draft.width * 2));
	} else if (draft.kind === 'connector') {
		ctx.strokeStyle = ctx.fillStyle = draft.color;
		strokePolyline(ctx, draft.points, 2, draft.head ? 'end' : 'none', CONNECTOR_RADIUS);
	} else {
		ctx.strokeStyle = palette.muted;
		ctx.globalAlpha = 0.4;
		strokePolyline(ctx, draft.points, 12 / camera.z, 'none');
	}
	ctx.restore();
}

/** Paints the overlay canvas: hover, selections (own and peers'), marquee, guides, drafts, bindings, cursors. */
export function drawOverlay(
	canvas: HTMLCanvasElement,
	scene: Scene,
	state: OverlayState,
	camera: Camera,
	palette: Palette
): void {
	const ctx = canvas.getContext('2d');
	if (!ctx) return;
	const { width, height, dpr } = fitCanvas(canvas);
	ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
	ctx.clearRect(0, 0, width, height);
	ctx.lineWidth = 1.5;
	for (const peer of state.peers) {
		ctx.strokeStyle = peer.color;
		for (const id of peer.selection) {
			const shape = scene.get(id);
			if (shape) outlineShape(ctx, shape, scene, camera);
		}
	}
	ctx.strokeStyle = palette.info;
	for (const id of new Set([...state.selection, ...(state.hover ? [state.hover] : [])])) {
		const shape = scene.get(id);
		if (shape) outlineShape(ctx, shape, scene, camera);
	}
	drawGuides(ctx, state, camera, { x: width, y: height });
	if (state.marquee) {
		const a = toScreen(camera, state.marquee);
		ctx.fillStyle = palette.surface;
		ctx.globalAlpha = 0.5;
		ctx.fillRect(a.x, a.y, state.marquee.w * camera.z, state.marquee.h * camera.z);
		ctx.globalAlpha = 1;
		ctx.strokeRect(a.x, a.y, state.marquee.w * camera.z, state.marquee.h * camera.z);
	}
	if (state.draft) drawDraft(ctx, state.draft, camera, palette);
	const target = state.target && scene.get(state.target.shape);
	if (target && state.target) {
		const p = toScreen(camera, anchorPoint(target, state.target.anchor));
		ctx.fillStyle = palette.info;
		outlineShape(ctx, target, scene, camera);
		ctx.beginPath();
		ctx.arc(p.x, p.y, 5, 0, Math.PI * 2);
		ctx.fill();
	}
	for (const peer of state.peers) drawCursor(ctx, peer, camera, palette);
}
