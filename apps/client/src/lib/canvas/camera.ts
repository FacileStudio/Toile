/** A point in world or screen space. */
export type Point = { x: number; y: number };

/** An axis-aligned box. */
export type Box = { x: number; y: number; w: number; h: number };

/** The view: `screen = world * z + (x, y)`. */
export type Camera = { x: number; y: number; z: number };

/** The zoom range, 5% to 800%. */
export const ZOOM = { min: 0.05, max: 8 } as const;

const clampZoom = (z: number) => Math.min(ZOOM.max, Math.max(ZOOM.min, z));

/** The world point under screen point `p`. */
export function toWorld(camera: Camera, p: Point): Point {
	return { x: (p.x - camera.x) / camera.z, y: (p.y - camera.y) / camera.z };
}

/** The screen point of world point `p`. */
export function toScreen(camera: Camera, p: Point): Point {
	return { x: p.x * camera.z + camera.x, y: p.y * camera.z + camera.y };
}

/** Zooms by `factor` keeping the world point under screen point `anchor` fixed. */
export function zoomAt(camera: Camera, anchor: Point, factor: number): Camera {
	const z = clampZoom(camera.z * factor);
	const ratio = z / camera.z;
	return { x: anchor.x - (anchor.x - camera.x) * ratio, y: anchor.y - (anchor.y - camera.y) * ratio, z };
}

/** Moves the view by a screen-space delta. */
export function panBy(camera: Camera, dx: number, dy: number): Camera {
	return { x: camera.x + dx, y: camera.y + dy, z: camera.z };
}

/** The world box a `width` x `height` screen shows. */
export function viewport(camera: Camera, width: number, height: number): Box {
	return { x: -camera.x / camera.z, y: -camera.y / camera.z, w: width / camera.z, h: height / camera.z };
}

/** The camera that shows world box `box` centred in a `width` x `height` screen with `pad` pixels of margin. */
export function fitBox(box: Box, width: number, height: number, pad = 48, maxZoom = 1): Camera {
	const z = clampZoom(
		Math.min(maxZoom, (width - pad * 2) / Math.max(box.w, 1), (height - pad * 2) / Math.max(box.h, 1))
	);
	return { x: width / 2 - (box.x + box.w / 2) * z, y: height / 2 - (box.y + box.h / 2) * z, z };
}

/** One pinch step: zoom by the finger-distance ratio around the midpoint, and pan by how far the midpoint moved. */
export function pinch(camera: Camera, from: [Point, Point], to: [Point, Point]): Camera {
	const a = { x: (from[0].x + from[1].x) / 2, y: (from[0].y + from[1].y) / 2 };
	const b = { x: (to[0].x + to[1].x) / 2, y: (to[0].y + to[1].y) / 2 };
	const before = Math.hypot(from[0].x - from[1].x, from[0].y - from[1].y) || 1;
	const after = Math.hypot(to[0].x - to[1].x, to[0].y - to[1].y) || 1;
	const zoomed = zoomAt(camera, a, after / before);
	return panBy(zoomed, b.x - a.x, b.y - a.y);
}
