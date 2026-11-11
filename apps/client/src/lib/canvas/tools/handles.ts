import type { Shape, ShapeFields } from '$lib/board/schema';
import type { Box, Point } from '../camera';
import type { Editor } from '../editor.svelte';
import { center, rotate, rotatedBox, union } from '../geometry';
import { normalizeAngle, resizeBox, rotationDelta, scaleInto, type Handle } from '../transform';
import type { PointerSample, Tool } from './tool';

function editableShapes(editor: Editor): Shape[] {
	return editor.selection.map((id) => editor.scene.get(id)).filter((s): s is Shape => !!s && !s.locked);
}

function fixedPoint(box: Box, handle: Handle): Point {
	return { x: handle.includes('w') ? box.x + box.w : box.x, y: handle.includes('n') ? box.y + box.h : box.y };
}

/** Resizes the selection from a handle: one shape in its own rotated frame, several as a group box. */
export class ResizeGesture implements Tool {
	cursor = 'nwse-resize';
	private shapes: Shape[];
	private group: Box | null;

	constructor(
		private editor: Editor,
		private handle: Handle,
		private start: Point
	) {
		this.shapes = editableShapes(editor);
		this.group = union(this.shapes.map((s) => rotatedBox(s)));
	}

	down(): void {}

	move(sample: PointerSample): void {
		const changes = new Map<string, ShapeFields>();
		if (this.shapes.length === 1) {
			const shape = this.shapes[0];
			changes.set(shape.id, this.single(shape, sample));
		} else if (this.group) {
			const dx = sample.world.x - this.start.x;
			const dy = sample.world.y - this.start.y;
			const to = resizeBox(this.group, this.handle, dx, dy, sample.shift);
			for (const shape of this.shapes) changes.set(shape.id, scaleInto(shape, this.group, to));
		}
		this.editor.doc.updateMany(changes);
	}

	private single(shape: Shape, sample: PointerSample): ShapeFields {
		const local = rotate(
			{ x: sample.world.x - this.start.x, y: sample.world.y - this.start.y },
			{ x: 0, y: 0 },
			-shape.rot
		);
		const keep = sample.shift || shape.type === 'image';
		const to = resizeBox(shape, this.handle, local.x, local.y, keep);
		const before = rotate(fixedPoint(shape, this.handle), center(shape), shape.rot);
		const after = rotate(fixedPoint(to, this.handle), center(to), shape.rot);
		return scaleInto(shape, shape, { ...to, x: to.x + before.x - after.x, y: to.y + before.y - after.y });
	}

	up(): void {}

	cancel(): void {}
}

/** Rotates the selection around its centre; Shift snaps to 15° steps. Arrows with bound ends stay. */
export class RotateGesture implements Tool {
	cursor = 'grabbing';
	private shapes: Shape[];
	private pivot: Point;

	constructor(
		private editor: Editor,
		private start: Point
	) {
		this.shapes = editableShapes(editor).filter((s) => !s.from && !s.to);
		const box = union(this.shapes.map((s) => rotatedBox(s)));
		this.pivot = box ? center(box) : start;
	}

	down(): void {}

	move(sample: PointerSample): void {
		const delta = rotationDelta(this.pivot, this.start, sample.world, sample.shift);
		const changes = new Map<string, ShapeFields>();
		for (const shape of this.shapes) {
			const c = rotate(center(shape), this.pivot, delta);
			changes.set(shape.id, { x: c.x - shape.w / 2, y: c.y - shape.h / 2, rot: normalizeAngle(shape.rot + delta) });
		}
		this.editor.doc.updateMany(changes);
	}

	up(): void {}

	cancel(): void {}
}
