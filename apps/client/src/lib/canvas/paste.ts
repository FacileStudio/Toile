import { toast } from '@facile/muse';
import { resolveCard } from '$lib/api/cards';
import { deleteSelection, createShape } from './actions';
import type { Point } from './camera';
import { insertShapes, parseClip, serialize } from './clipboard';
import type { Editor } from './editor.svelte';
import { youtubeId, youtubeThumbnail } from './youtube';

/** Writes the selection to the clipboard; `cut` also deletes it. Returns whether anything was copied. */
export function copyOut(editor: Editor, data: DataTransfer, cut: boolean): boolean {
	const text = serialize(editor);
	if (!text) return false;
	data.setData('text/plain', text);
	if (cut) deleteSelection(editor);
	return true;
}

const CLIP = /^\s*\{\s*"type"\s*:\s*"toile\/shapes"/;

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

async function fillCard(editor: Editor, id: string, url: string): Promise<void> {
	const card = await resolveCard(url).catch(() => null);
	if (!card) return;
	const shape = editor.scene.shapes.get(id);
	if (!shape) return;
	const h = card.image && shape.h < 180 ? 180 : undefined;
	const youtube = shape.kind === 'youtube';
	editor.doc.update(id, {
		title: youtube ? shape.title : card.title,
		kind: youtube ? 'youtube' : card.kind,
		image: card.image ?? shape.image,
		description: card.subtitle,
		favicon: card.favicon,
		site: card.site ?? shape.site,
		...(h !== undefined ? { h } : {})
	});
}

function addLink(editor: Editor, trimmed: string, at: Point): string | null {
	const video = youtubeId(trimmed);
	if (video) {
		return createShape(editor, 'card', {
			x: at.x - 160,
			y: at.y - 100,
			w: 320,
			h: 200,
			kind: 'youtube',
			url: trimmed,
			title: 'Vidéo YouTube',
			image: youtubeThumbnail(video),
			site: 'YouTube'
		});
	}
	let hostname = trimmed;
	try {
		hostname = new URL(trimmed).hostname;
	} catch {}
	return createShape(editor, 'card', {
		x: at.x - 160,
		y: at.y - 50,
		w: 320,
		h: 100,
		kind: 'link',
		url: trimmed,
		title: hostname
	});
}

function addText(editor: Editor, text: string, at: Point): void {
	const trimmed = text.trim();
	if (!trimmed) return;
	let id: string | null = null;
	if (/^https?:\/\/\S+\.(png|jpe?g|gif|webp|svg)(\?\S*)?$/i.test(trimmed)) {
		id = createShape(editor, 'image', { x: at.x - 160, y: at.y - 120, w: 320, h: 240, asset: trimmed });
	} else if (/^https?:\/\/\S+$/.test(trimmed)) {
		id = addLink(editor, trimmed, at);
		if (id) void fillCard(editor, id, trimmed);
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
	if (!shapes && CLIP.test(text)) {
		toast.danger('Ces éléments copiés sont illisibles : copiez-les à nouveau.');
		return;
	}
	if (!shapes) return addText(editor, text, point);
	const local = shapes.every((shape) => editor.scene.shapes.has(shape.id));
	insertShapes(editor, shapes, local && !at ? undefined : point);
}
