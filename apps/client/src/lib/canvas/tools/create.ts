import type { ShapeType } from '$lib/board/schema';
import { createShape, styleFor } from '../actions';
import type { Box, Point } from '../camera';
import type { Editor } from '../editor.svelte';
import { boxOf } from '../geometry';
import { frameAt } from '../hit';
import { STICKY_FILL } from '../render/palette';
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

/**
 * Places a box shape by click (default size, centred) or by drag (the dragged box), then selects it. A sticky shows a
 * translucent preview under the pointer before the click and stays square when dragged, as in FigJam.
 */
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
		this.preview(null);
	}

	hover(sample: PointerSample): void {
		if (this.type !== 'sticky') return;
		const [w, h] = DEFAULT_SIZE.sticky ?? [200, 200];
		this.preview({ x: sample.world.x - w / 2, y: sample.world.y - h / 2, w, h });
	}

	leave(): void {
		this.preview(null);
	}

	move(sample: PointerSample): void {
		if (!this.start) return;
		const dragged = Math.hypot(sample.world.x - this.start.x, sample.world.y - this.start.y) * this.editor.camera.z;
		if (dragged < 6) return;
		this.box = boxOf([this.start, sample.world]);
		if (sample.shift || this.type === 'sticky') this.square(this.box, sample.world);
		if (this.type === 'sticky') this.preview(this.box);
		else this.editor.overlay.marquee = this.box;
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
		this.preview(null);
	}

	private square(box: Box, to: Point): void {
		const side = Math.max(box.w, box.h);
		if (this.start && to.x < this.start.x) box.x = this.start.x - side;
		if (this.start && to.y < this.start.y) box.y = this.start.y - side;
		box.w = box.h = side;
	}

	private preview(box: Box | null): void {
		const color = styleFor('sticky', this.editor.style).fill ?? STICKY_FILL;
		this.editor.overlay.draft = box ? { kind: 'box', box, color, radius: 4 } : null;
		this.editor.invalidateOverlay();
	}
}
