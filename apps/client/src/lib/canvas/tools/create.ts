import type { ShapeType } from '$lib/board/schema';
import { createShape } from '../actions';
import type { Box, Point } from '../camera';
import type { Editor } from '../editor.svelte';
import { boxOf } from '../geometry';
import { frameAt } from '../hit';
import type { PointerSample, Tool } from './tool';

/** The size a click (no drag) gives each box type, in world units. */
export const DEFAULT_SIZE: Partial<Record<ShapeType, [number, number]>> = {
	sticky: [200, 200],
	text: [240, 32],
	rect: [180, 120],
	ellipse: [160, 160],
	diamond: [180, 140],
	frame: [800, 520]
};

/** Places a box shape by click (default size, centred) or by drag (the dragged box), then selects it. */
export class CreateTool implements Tool {
	cursor = 'crosshair';
	private start: Point | null = null;
	private box: Box | null = null;

	constructor(
		private editor: Editor,
		private type: ShapeType
	) {}

	down(sample: PointerSample): void {
		this.start = sample.world;
		this.box = null;
	}

	move(sample: PointerSample): void {
		if (!this.start) return;
		const dragged = Math.hypot(sample.world.x - this.start.x, sample.world.y - this.start.y) * this.editor.camera.z;
		if (dragged < 6) return;
		this.box = boxOf([this.start, sample.world]);
		if (sample.shift) this.box.w = this.box.h = Math.max(this.box.w, this.box.h);
		this.editor.overlay.marquee = this.box;
		this.editor.invalidateOverlay();
	}

	up(): void {
		if (!this.start) return;
		const [w, h] = DEFAULT_SIZE[this.type] ?? [160, 120];
		const box = this.box ?? { x: this.start.x - w / 2, y: this.start.y - (this.type === 'text' ? h / 2 : h / 2), w, h };
		this.cancel();
		const parent =
			this.type === 'frame'
				? null
				: frameAt(this.editor.scene, { x: box.x + box.w / 2, y: box.y + box.h / 2 }, new Set());
		const id = createShape(this.editor, this.type, {
			...box,
			parent: parent?.id,
			...(this.type === 'frame' ? { name: 'Cadre' } : {})
		});
		if (!id) return;
		this.editor.tool = 'select';
		this.editor.select([id]);
		if (this.type !== 'frame') this.editor.editing = id;
	}

	cancel(): void {
		this.start = null;
		this.box = null;
		this.editor.overlay.marquee = null;
		this.editor.invalidateOverlay();
	}
}
