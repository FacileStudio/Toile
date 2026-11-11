import { pathPoints } from '../bounds';
import type { Editor } from '../editor.svelte';
import { boxOf } from '../geometry';
import { bindPoint } from './connector';
import type { PointerSample, Tool } from './tool';

/** Drags one end of a line or an arrow, binding it to the shape it lands on or freeing it on empty board. */
export class EndpointGesture implements Tool {
	cursor = 'crosshair';

	constructor(
		private editor: Editor,
		private id: string,
		private end: 'from' | 'to'
	) {}

	down(): void {}

	move(sample: PointerSample): void {
		const arrow = this.editor.scene.get(this.id);
		if (!arrow || arrow.locked) return;
		const other = this.end === 'from' ? arrow.to?.shape : arrow.from?.shape;
		const target = bindPoint(this.editor, sample.world, other);
		const points = pathPoints(arrow, this.editor.scene.get);
		points[this.end === 'from' ? 0 : points.length - 1] = target.point;
		const box = boxOf(points);
		this.editor.doc.update(this.id, {
			x: box.x,
			y: box.y,
			w: box.w,
			h: box.h,
			rot: 0,
			points: points.flatMap((p) => [p.x - box.x, p.y - box.y]),
			[this.end]: target.binding
		});
		this.editor.overlay.target = target.binding ?? null;
		this.editor.invalidateOverlay();
	}

	up(): void {
		this.cancel();
	}

	cancel(): void {
		this.editor.overlay.target = null;
		this.editor.invalidateOverlay();
	}
}
