import { TEXT_TYPES, type Shape } from '$lib/board/schema';
import type { Point } from '../camera';
import type { Editor } from '../editor.svelte';
import { boxOf } from '../geometry';
import { marqueeHits, topHit } from '../hit';
import { MoveGesture } from './move';
import type { PointerSample, Tool } from './tool';

/** Opens a shape for editing: text types edit their text, frames their name, cards open their link. */
export function openShape(editor: Editor, shape: Shape): void {
	if (shape.type === 'card' && shape.url) {
		window.open(shape.url, '_blank', 'noopener');
		return;
	}
	if (!editor.editable || shape.locked) return;
	if (TEXT_TYPES.has(shape.type) || shape.type === 'frame') {
		editor.select([shape.id]);
		editor.editing = shape.id;
	}
}

/** Clicks select, Shift toggles, a drag on a shape moves the selection, a drag on empty board draws a marquee. */
export class SelectTool implements Tool {
	cursor = 'default';
	private start: Point | null = null;
	private base: string[] = [];
	private marquee = false;
	private gesture: MoveGesture | null = null;
	private pressed: PointerSample | null = null;

	constructor(private editor: Editor) {}

	down(sample: PointerSample): void {
		const editor = this.editor;
		if (editor.editing && sample.detail < 2) editor.finishEditing();
		const hit = topHit(editor.scene, sample.world, 4 / editor.camera.z);
		this.start = sample.world;
		this.pressed = sample;
		if (hit && sample.detail >= 2) return openShape(editor, hit);
		if (!hit) {
			this.marquee = true;
			this.base = sample.shift ? editor.selection : [];
			if (!sample.shift) editor.select([]);
			return;
		}
		if (sample.shift) {
			const has = editor.selection.includes(hit.id);
			editor.select(has ? editor.selection.filter((id) => id !== hit.id) : [...editor.selection, hit.id]);
		} else if (!editor.selection.includes(hit.id)) editor.select([hit.id]);
	}

	move(sample: PointerSample): void {
		if (!this.start) return this.hover(sample);
		if (this.marquee) return this.drawMarquee(sample);
		if (this.gesture) return this.gesture.move(sample);
		const moved = Math.hypot(sample.world.x - this.start.x, sample.world.y - this.start.y) * this.editor.camera.z;
		if (moved > 3 && this.editor.editable && this.pressed && this.editor.selection.length) {
			this.gesture = new MoveGesture(this.editor, this.pressed);
			this.gesture.move(sample);
		}
	}

	hover(sample: PointerSample): void {
		const hit = topHit(this.editor.scene, sample.world, 4 / this.editor.camera.z);
		if ((hit?.id ?? null) === this.editor.overlay.hover) return;
		this.editor.overlay.hover = hit?.id ?? null;
		this.editor.invalidateOverlay();
	}

	up(): void {
		this.gesture?.up();
		this.reset();
	}

	cancel(): void {
		this.gesture?.cancel();
		this.reset();
	}

	private drawMarquee(sample: PointerSample): void {
		if (!this.start) return;
		const box = boxOf([this.start, sample.world]);
		this.editor.overlay.marquee = box;
		const ids = marqueeHits(this.editor.scene, box).map((shape) => shape.id);
		this.editor.select([...new Set([...this.base, ...ids])]);
	}

	private reset(): void {
		this.start = null;
		this.pressed = null;
		this.marquee = false;
		this.gesture = null;
		this.editor.overlay.marquee = null;
		this.editor.invalidateOverlay();
	}
}
