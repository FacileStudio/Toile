import { deleteSelection, createShape } from './actions';
import type { Point } from './camera';
import { insertShapes, parseClip, serialize } from './clipboard';
import type { Editor } from './editor.svelte';

/** Writes the selection to the clipboard; `cut` also deletes it. Returns whether anything was copied. */
export function copyOut(editor: Editor, data: DataTransfer, cut: boolean): boolean {
	const text = serialize(editor);
	if (!text) return false;
	data.setData('text/plain', text);
	if (cut) deleteSelection(editor);
	return true;
}

function viewCentre(editor: Editor): Point {
	return editor.world({ x: editor.size.width / 2, y: editor.size.height / 2 });
}

async function addImage(editor: Editor, file: File, at: Point): Promise<void> {
	const uploaded = await editor.host.uploadImage(file);
	if (!uploaded) return;
	const scale = Math.min(1, 480 / Math.max(uploaded.w, uploaded.h, 1));
	const w = uploaded.w * scale;
	const h = uploaded.h * scale;
	const id = createShape(editor, 'image', { x: at.x - w / 2, y: at.y - h / 2, w, h, asset: uploaded.asset });
	if (id) editor.select([id]);
}

function addText(editor: Editor, text: string, at: Point): void {
	const trimmed = text.trim();
	if (!trimmed) return;
	let id: string | null;
	if (/^https?:\/\/\S+$/.test(trimmed)) {
		const title = new URL(trimmed).hostname;
		id = createShape(editor, 'card', { x: at.x - 160, y: at.y - 40, w: 320, h: 80, url: trimmed, title, kind: 'link' });
	} else {
		id = createShape(editor, 'text', { x: at.x - 160, y: at.y - 16, w: 320, h: 32 }, trimmed);
	}
	if (id) editor.select([id]);
}

/**
 * Pastes or drops clipboard data at `at` (the view centre when null): image files go through the upload hook,
 * Toile shapes are copied (offset in place when they come from this board), a link becomes a card, text a text shape.
 */
export async function pasteInto(editor: Editor, data: DataTransfer, at: Point | null): Promise<void> {
	if (!editor.editable) return;
	const point = at ?? viewCentre(editor);
	const images = [...data.files].filter((file) => file.type.startsWith('image/'));
	if (images.length) {
		for (const file of images) await addImage(editor, file, point);
		return;
	}
	const text = data.getData('text/plain');
	const shapes = parseClip(text);
	if (!shapes) return addText(editor, text, point);
	const local = shapes.every((shape) => editor.scene.shapes.has(shape.id));
	insertShapes(editor, shapes, local && !at ? undefined : point);
}
