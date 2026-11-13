import type { Binding, Shape } from '$lib/board/schema';
import { createShape } from '../actions';
import { anchorOf, anchorPoint, elbow, type End } from '../anchors';
import type { Point } from '../camera';
import type { Editor } from '../editor.svelte';
import { topHit } from '../hit';
import type { PointerSample, Tool } from './tool';

const SNAPS: [number, number][] = [
	[0.5, 0],
	[1, 0.5],
	[0.5, 1],
	[0, 0.5],
	[0.5, 0.5]
];

/** The anchor of `p` on `shape`, snapped to the nearest cardinal anchor when within 35% of one. */
export function snapAnchor(shape: Shape, p: Point): [number, number] {
	const anchor = anchorOf(shape, p);
	let best = anchor;
	let min = Infinity;
	for (const snap of SNAPS) {
		const d = Math.hypot(snap[0] - anchor[0], snap[1] - anchor[1]);
		if (d < min) {
			min = d;
			best = snap;
		}
	}
	return min < 0.35 ? best : anchor;
}

/** The point and binding an arrow end takes at world point `p`: the anchor of the shape under it, if any. */
export function bindPoint(editor: Editor, p: Point, skip?: string): { point: Point; binding?: Binding } {
	const tolerance = 6 / editor.camera.z;
	const shape = topHit(
		editor.scene,
		p,
		tolerance,
		(s) => s.id === skip || s.type === 'arrow' || s.type === 'line' || s.type === 'ink'
	);
	if (!shape) return { point: p };
	const anchor = snapAnchor(shape, p);
	return { point: anchorPoint(shape, anchor), binding: { shape: shape.id, anchor } };
}

/** Draws an arrow or a line from a point or a shape to a point or a shape; bound ends follow their shapes. */
export class ConnectorTool implements Tool {
	cursor = 'crosshair';
	private from: { point: Point; binding?: Binding } | null = null;

	constructor(
		private editor: Editor,
		private type: 'arrow' | 'line' = 'arrow',
		initialFrom?: { point: Point; binding?: Binding }
	) {
		if (initialFrom) this.from = initialFrom;
	}

	private bindAt(p: Point, skip?: string): { point: Point; binding?: Binding } {
		return bindPoint(this.editor, p, skip);
	}

	down(sample: PointerSample): void {
		if (!this.from) this.from = this.bindAt(sample.world);
	}

	move(sample: PointerSample): void {
		if (!this.from) return this.hover(sample);
		const to = this.bindAt(sample.world, this.from.binding?.shape);
		const color = this.editor.style.stroke ?? this.editor.palette.fg;
		this.editor.overlay.draft = {
			kind: 'connector',
			points: this.route(this.from, to),
			color,
			head: this.type === 'arrow'
		};
		this.editor.overlay.target = to.binding ?? null;
		this.editor.invalidateOverlay();
	}

	private route(from: { point: Point; binding?: Binding }, to: { point: Point; binding?: Binding }): Point[] {
		if (!from.binding && !to.binding) return [from.point, to.point];
		const end = (side: { point: Point; binding?: Binding }): End => ({
			point: side.point,
			shape: side.binding && this.editor.scene.get(side.binding.shape),
			anchor: side.binding?.anchor
		});
		return elbow(end(from), end(to));
	}

	hover(sample: PointerSample): void {
		this.editor.overlay.target = this.bindAt(sample.world).binding ?? null;
		this.editor.invalidateOverlay();
	}

	up(sample: PointerSample): void {
		const from = this.from;
		this.cancel();
		if (!from) return;
		const to = this.bindAt(sample.world, from.binding?.shape);
		if (Math.hypot(to.point.x - from.point.x, to.point.y - from.point.y) * this.editor.camera.z < 8) return;
		const x = Math.min(from.point.x, to.point.x);
		const y = Math.min(from.point.y, to.point.y);
		const points = [from.point.x - x, from.point.y - y, to.point.x - x, to.point.y - y];
		const w = Math.abs(to.point.x - from.point.x);
		const h = Math.abs(to.point.y - from.point.y);
		const head = this.type === 'arrow' ? { head: 'end' as const } : {};
		const id = createShape(this.editor, this.type, { x, y, w, h, points, from: from.binding, to: to.binding, ...head });
		if (!id) return;
		this.editor.tool = 'select';
		this.editor.select([id]);
	}

	cancel(): void {
		this.from = null;
		this.editor.overlay.draft = null;
		this.editor.overlay.target = null;
		this.editor.invalidateOverlay();
	}
}
