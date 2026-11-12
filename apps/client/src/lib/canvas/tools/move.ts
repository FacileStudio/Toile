import type { ShapeFields } from '$lib/board/schema';
import { viewport, type Box, type Point } from '../camera';
import type { Editor } from '../editor.svelte';
import { center, rotatedBox, union } from '../geometry';
import { frameAt } from '../hit';
import { snapBox } from '../snap';
import type { PointerSample, Tool } from './tool';

const LIFT_LIMIT = 50;

/** The ids a drag of `ids` moves: the unlocked ones plus every shape inside a moved frame. */
export function movingIds(editor: Editor, ids: readonly string[]): string[] {
	const moving = new Set(ids.filter((id) => !editor.scene.get(id)?.locked));
	const frames = new Set([...moving].filter((id) => editor.scene.get(id)?.type === 'frame'));
	if (frames.size)
		for (const shape of editor.scene.shapes.values())
			if (shape.parent && frames.has(shape.parent) && !shape.locked) moving.add(shape.id);
	return [...moving];
}

/** Moves the selection (and frame contents) with the pointer, snapping to nearby shapes unless Alt is held. */
export class MoveGesture implements Tool {
	cursor = 'move';
	private origins = new Map<string, Point>();
	private start: Point;
	private box: Box | null;
	private others: Box[];

	constructor(
		private editor: Editor,
		sample: PointerSample
	) {
		this.start = sample.world;
		const ids = movingIds(editor, editor.selection);
		for (const id of ids) {
			const shape = editor.scene.get(id);
			if (shape) this.origins.set(id, { x: shape.x, y: shape.y });
		}
		this.box = union(ids.map((id) => editor.scene.index.box(id)).filter((b): b is Box => !!b));
		const view = viewport(editor.camera, editor.size.width, editor.size.height);
		editor.lift(ids.length <= LIFT_LIMIT ? ids : []);
		this.others = editor.scene
			.within(view)
			.filter(
				(shape) =>
					!this.origins.has(shape.id) && shape.type !== 'ink' && shape.type !== 'arrow' && shape.type !== 'line'
			)
			.slice(-300)
			.map((shape) => rotatedBox(shape));
	}

	down(): void {}

	move(sample: PointerSample): void {
		let dx = sample.world.x - this.start.x;
		let dy = sample.world.y - this.start.y;
		const guides = { x: [] as number[], y: [] as number[] };
		if (this.box && !sample.alt) {
			const snap = snapBox(
				{ ...this.box, x: this.box.x + dx, y: this.box.y + dy },
				this.others,
				6 / this.editor.camera.z
			);
			dx += snap.dx;
			dy += snap.dy;
			Object.assign(guides, snap.guides);
		}
		const changes = new Map<string, ShapeFields>();
		this.origins.forEach((o, id) => changes.set(id, { x: o.x + dx, y: o.y + dy }));
		this.editor.doc.updateMany(changes);
		this.editor.overlay.guides = guides;
		this.editor.invalidateOverlay();
	}

	up(): void {
		this.reparent();
		this.cancel();
	}

	cancel(): void {
		this.editor.lift([]);
		this.editor.overlay.guides = { x: [], y: [] };
		this.editor.invalidateOverlay();
	}

	private reparent(): void {
		this.editor.sync();
		const moved = new Set(this.origins.keys());
		const changes = new Map<string, ShapeFields>();
		for (const id of this.editor.selection) {
			const shape = this.editor.scene.get(id);
			if (!shape || shape.type === 'frame' || !moved.has(id)) continue;
			const frame = frameAt(this.editor.scene, center(rotatedBox(shape)), moved);
			if ((frame?.id ?? undefined) !== shape.parent) changes.set(id, { parent: frame?.id });
		}
		if (changes.size) this.editor.doc.updateMany(changes);
	}
}
