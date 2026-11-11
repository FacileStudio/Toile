import { keysBetween } from '$lib/board/fractional';
import type { Shape } from '$lib/board/schema';
import type { Editor } from './editor.svelte';
import { intersects } from './geometry';

/** A z-order move. */
export type OrderMove = 'front' | 'back' | 'forward' | 'backward';

function chosen(editor: Editor): Shape[] {
	const ids = new Set(editor.selection);
	return editor.scene.order.filter((shape) => ids.has(shape.id) && !shape.locked);
}

function place(editor: Editor, shapes: Shape[], below: string | null, above: string | null): void {
	const keys = keysBetween(below, above, shapes.length);
	editor.doc.updateMany(new Map(shapes.map((shape, i) => [shape.id, { z: keys[i] }])));
}

function neighbour(editor: Editor, shapes: Shape[], up: boolean): Shape | null {
	const order = editor.scene.order;
	const ids = new Set(shapes.map((s) => s.id));
	const edge = up ? shapes[shapes.length - 1] : shapes[0];
	const boxes = shapes.map((s) => editor.scene.index.box(s.id)).filter((b) => !!b);
	const step = up ? 1 : -1;
	for (let i = editor.scene.rank(edge.id) + step; i >= 0 && i < order.length; i += step) {
		const other = order[i];
		const box = editor.scene.index.box(other.id);
		if (!ids.has(other.id) && box && boxes.some((b) => intersects(b, box))) return other;
	}
	return null;
}

/**
 * Restacks the unlocked selection, keeping its internal order: to the very front or back, or one step past the next
 * overlapping shape above or below it.
 */
export function restack(editor: Editor, move: OrderMove): void {
	const shapes = chosen(editor);
	if (!editor.editable || !shapes.length) return;
	const order = editor.scene.order;
	const top = order[order.length - 1]?.z ?? null;
	const bottom = order[0]?.z ?? null;
	if (move === 'front') return place(editor, shapes, top, null);
	if (move === 'back') return place(editor, shapes, null, bottom);
	const other = neighbour(editor, shapes, move === 'forward');
	if (!other) return;
	const rank = editor.scene.rank(other.id);
	if (move === 'forward') place(editor, shapes, other.z, order[rank + 1]?.z ?? null);
	else place(editor, shapes, order[rank - 1]?.z ?? null, other.z);
}
