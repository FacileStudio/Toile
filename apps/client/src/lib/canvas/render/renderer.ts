import { PATH_TYPES } from '$lib/board/schema';
import type { Camera } from '../camera';
import { viewport } from '../camera';
import type { Scene } from '../scene';
import type { Palette } from './palette';
import { drawShape, type DrawEnv } from './shapes';

const GRID = 24;
const COARSE = 0.15;

/** Sizes a canvas's backing store to its CSS box at the device pixel ratio. Returns the CSS size. */
export function fitCanvas(canvas: HTMLCanvasElement): { width: number; height: number; dpr: number } {
	const dpr = Math.min(window.devicePixelRatio || 1, 2);
	const width = canvas.clientWidth;
	const height = canvas.clientHeight;
	if (canvas.width !== Math.round(width * dpr) || canvas.height !== Math.round(height * dpr)) {
		canvas.width = Math.round(width * dpr);
		canvas.height = Math.round(height * dpr);
	}
	return { width, height, dpr };
}

function drawGrid(ctx: CanvasRenderingContext2D, camera: Camera, width: number, height: number, color: string): void {
	let step = GRID * camera.z;
	while (step < 12) step *= 4;
	const size = Math.min(2, Math.max(1, camera.z * 1.5));
	const startX = (((camera.x % step) + step) % step) - size / 2;
	const startY = (((camera.y % step) + step) % step) - size / 2;
	ctx.fillStyle = color;
	ctx.beginPath();
	for (let x = startX; x < width; x += step) {
		for (let y = startY; y < height; y += step) ctx.rect(x, y, size, size);
	}
	ctx.fill();
}

/** What one scene frame needs: the camera, the palette, the shape being edited and the image source. */
export type FrameInput = {
	camera: Camera;
	palette: Palette;
	editing: string | null;
	image: DrawEnv['image'];
};

/**
 * Paints the scene canvas: background, dot grid, then every shape the viewport overlaps, back to front. Under 15%
 * zoom, and for any box under two screen pixels, an unrotated box shape is one filled rectangle.
 */
export function drawScene(canvas: HTMLCanvasElement, scene: Scene, input: FrameInput): number {
	const ctx = canvas.getContext('2d');
	if (!ctx) return 0;
	const { width, height, dpr } = fitCanvas(canvas);
	const { camera, palette } = input;
	ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
	ctx.fillStyle = palette.page;
	ctx.fillRect(0, 0, width, height);
	drawGrid(ctx, camera, width, height, palette.surface);
	ctx.setTransform(dpr * camera.z, 0, 0, dpr * camera.z, dpr * camera.x, dpr * camera.y);
	const visible = scene.within(viewport(camera, width, height));
	const env: DrawEnv = { palette, lookup: scene.get, zoom: camera.z, editing: input.editing, image: input.image };
	const tiny = 2 / camera.z;
	const coarse = camera.z < COARSE;
	for (const shape of visible) {
		const boxy = !PATH_TYPES.has(shape.type) && shape.type !== 'frame';
		if (boxy && (coarse || (shape.w < tiny && shape.h < tiny)) && !shape.rot) {
			ctx.fillStyle = shape.style.fill ?? palette.surface;
			ctx.fillRect(shape.x, shape.y, Math.max(shape.w, tiny / 2), Math.max(shape.h, tiny / 2));
			continue;
		}
		drawShape(ctx, shape, env);
	}
	return visible.length;
}
