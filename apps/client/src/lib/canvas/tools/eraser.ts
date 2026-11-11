import type { Point } from '../camera';
import type { Editor } from '../editor.svelte';
import { hitShape } from '../hit';
import type { PointerSample, Tool } from './tool';

/** Erases the unlocked ink strokes the pointer passes over; one drag is one undo step. */
export class EraserTool implements Tool {
	cursor = 'cell';
	private trail: Point[] | null = null;

	constructor(private editor: Editor) {}

	down(sample: PointerSample): void {
		this.trail = [];
		this.move(sample);
	}

	move(sample: PointerSample): void {
		if (!this.trail) return;
		this.trail.push(sample.world);
		if (this.trail.length > 24) this.trail.shift();
		const tolerance = 8 / this.editor.camera.z;
		const area = { x: sample.world.x - tolerance, y: sample.world.y - tolerance, w: tolerance * 2, h: tolerance * 2 };
		const hits = this.editor.scene
			.within(area)
			.filter(
				(shape) =>
					shape.type === 'ink' && !shape.locked && hitShape(shape, sample.world, tolerance, this.editor.scene.get)
			);
		if (hits.length) this.editor.doc.remove(hits.map((shape) => shape.id));
		this.editor.overlay.draft = { kind: 'eraser', points: [...this.trail] };
		this.editor.invalidateOverlay();
	}

	up(): void {
		this.cancel();
		this.editor.undo.stopCapturing();
	}

	cancel(): void {
		this.trail = null;
		this.editor.overlay.draft = null;
		this.editor.invalidateOverlay();
	}
}
