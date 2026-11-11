import { fitBox, zoomAt, type Box } from './camera';
import type { Editor } from './editor.svelte';
import { union } from './geometry';

/** Zooms by `factor` around the centre of the screen. */
export function zoomBy(editor: Editor, factor: number): void {
	const middle = { x: editor.size.width / 2, y: editor.size.height / 2 };
	editor.setCamera(zoomAt(editor.camera, middle, factor));
}

/** Back to 100% around the centre of the screen. */
export function zoomReset(editor: Editor): void {
	zoomBy(editor, 1 / editor.camera.z);
}

/** Frames world box `box`, never zooming in past 100%. */
export function showBox(editor: Editor, box: Box, maxZoom = 1): void {
	editor.setCamera(fitBox(box, editor.size.width, editor.size.height, 64, maxZoom));
}

/** Frames every shape on the board, or the origin when it is empty. */
export function zoomFit(editor: Editor): void {
	const boxes = [...editor.scene.shapes.keys()].map((id) => editor.scene.index.box(id)).filter((b): b is Box => !!b);
	const box = union(boxes);
	if (box) showBox(editor, box);
	else editor.setCamera({ x: editor.size.width / 2, y: editor.size.height / 2, z: 1 });
}

/** Frames the selection, or the whole board when nothing is selected. */
export function zoomSelection(editor: Editor): void {
	const box = union(editor.selection.map((id) => editor.scene.index.box(id)).filter((b): b is Box => !!b));
	if (box) showBox(editor, box, 2);
	else zoomFit(editor);
}

/** Moves the unlocked selection by a world delta. */
export function nudge(editor: Editor, dx: number, dy: number): void {
	if (!editor.editable) return;
	const changes = new Map<string, { x: number; y: number }>();
	for (const id of editor.selection) {
		const shape = editor.scene.get(id);
		if (shape && !shape.locked) changes.set(id, { x: shape.x + dx, y: shape.y + dy });
	}
	editor.doc.updateMany(changes);
}
