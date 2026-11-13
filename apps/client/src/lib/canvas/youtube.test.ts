import { afterEach, describe, expect, it, vi } from 'vitest';
import { BoardDoc } from '$lib/board/doc';
import type { Shape } from '$lib/board/schema';
import type { Editor } from './editor.svelte';
import { pasteInto } from './paste';
import { Scene } from './scene';
import { SelectTool } from './tools/select';
import { youtubeEmbed, youtubeId } from './youtube';

afterEach(() => vi.unstubAllGlobals());

describe('youtubeId', () => {
	it.each([
		['https://youtu.be/PAy5EBTnV6Q?list=RDERCkf0j8pPM', 'PAy5EBTnV6Q'],
		['https://youtu.be/PAy5EBTnV6Q?si=x&t=42', 'PAy5EBTnV6Q'],
		['https://www.youtube.com/watch?v=PAy5EBTnV6Q&list=RDERCkf0j8pPM&index=2', 'PAy5EBTnV6Q'],
		['https://m.youtube.com/watch?feature=share&v=PAy5EBTnV6Q', 'PAy5EBTnV6Q'],
		['https://music.youtube.com/watch?v=PAy5EBTnV6Q', 'PAy5EBTnV6Q'],
		['https://www.youtube.com/shorts/PAy5EBTnV6Q?feature=share', 'PAy5EBTnV6Q'],
		['https://www.youtube.com/live/PAy5EBTnV6Q', 'PAy5EBTnV6Q'],
		['https://www.youtube-nocookie.com/embed/PAy5EBTnV6Q', 'PAy5EBTnV6Q'],
		['  https://youtu.be/PAy5EBTnV6Q  ', 'PAy5EBTnV6Q']
	])('reads %s', (raw, id) => expect(youtubeId(raw)).toBe(id));

	it.each([
		'https://youtu.be/',
		'https://www.youtube.com/watch?v=short',
		'https://www.youtube.com/feed/explore',
		'https://notyoutube.com/watch?v=PAy5EBTnV6Q',
		'https://example.com/?u=youtube.com/watch?v=PAy5EBTnV6Q',
		'javascript:alert("youtu.be/PAy5EBTnV6Q")',
		'not a url'
	])('refuses %s', (raw) => expect(youtubeId(raw)).toBeNull());
});

describe('youtubeEmbed', () => {
	it('builds the nocookie player with autoplay and the start offset', () => {
		expect(youtubeEmbed('https://youtu.be/PAy5EBTnV6Q?list=RDERCkf0j8pPM')).toBe(
			'https://www.youtube-nocookie.com/embed/PAy5EBTnV6Q?autoplay=1&rel=0'
		);
		expect(youtubeEmbed('https://youtu.be/PAy5EBTnV6Q?t=1m30s')).toBe(
			'https://www.youtube-nocookie.com/embed/PAy5EBTnV6Q?autoplay=1&rel=0&start=90'
		);
		expect(youtubeEmbed('https://www.youtube.com/watch?v=PAy5EBTnV6Q&t=42')).toContain('start=42');
		expect(youtubeEmbed('https://example.com')).toBeNull();
	});
});

describe('youtube cards', () => {
	function pasteEditor(doc: BoardDoc) {
		return {
			editable: true,
			world: (p: { x: number; y: number }) => p,
			size: { width: 800, height: 600 },
			scene: { shapes: doc.shapes, order: [] as Shape[] },
			doc,
			select: vi.fn(),
			host: { uploadImage: vi.fn() }
		};
	}

	function clip(text: string) {
		return { files: [], getData: (t: string) => (t === 'text/plain' ? text : '') };
	}

	it.each([
		'https://youtu.be/PAy5EBTnV6Q?list=RDERCkf0j8pPM',
		'https://www.youtube.com/watch?v=PAy5EBTnV6Q&t=42s',
		'https://www.youtube.com/shorts/PAy5EBTnV6Q'
	])('pastes %s as a playable youtube card', async (url) => {
		const doc = new BoardDoc();
		await pasteInto(pasteEditor(doc) as unknown as Editor, clip(url) as unknown as DataTransfer, { x: 0, y: 0 });
		const [card] = doc.all();
		expect(card).toMatchObject({ type: 'card', kind: 'youtube', url, site: 'YouTube' });
		expect(card.image).toBe('https://img.youtube.com/vi/PAy5EBTnV6Q/hqdefault.jpg');
	});

	it('pastes copied toile shapes with their card preview fields and never as text', async () => {
		const doc = new BoardDoc();
		const payload =
			'{"type":"toile/shapes","version":1,"shapes":[{"id":"ceS6wM59r4pIHwfY","type":"card","x":-1652.25,"y":-877.009765625,"w":320,"h":200,"rot":0,"z":"Y","style":{},"kind":"youtube","url":"https://youtu.be/PAy5EBTnV6Q?list=RDERCkf0j8pPM","title":"youtu.be","image":"https://img.youtube.com/vi/PAy5EBTnV6Q/hqdefault.jpg","favicon":"https://www.google.com/s2/favicons?domain=youtu.be&sz=64","site":"YouTube"}]}';
		const editor = pasteEditor(doc);
		await pasteInto(editor as unknown as Editor, clip(payload) as unknown as DataTransfer, { x: 0, y: 0 });
		const [card] = doc.all();
		expect(card).toMatchObject({ type: 'card', kind: 'youtube', site: 'YouTube' });
		expect(card.favicon).toContain('favicons');
		await pasteInto(editor as unknown as Editor, clip(payload.slice(0, 80)) as unknown as DataTransfer, null);
		expect(doc.all()).toHaveLength(1);
	});

	it('plays a youtube card in place when its play button is clicked', () => {
		const open = vi.fn();
		vi.stubGlobal('open', open);
		const doc = new BoardDoc();
		const scene = new Scene(doc, () => undefined);
		const id = doc.create('card', {
			x: 100,
			y: 100,
			w: 320,
			h: 200,
			z: 'V',
			kind: 'youtube',
			url: 'https://youtu.be/PAy5EBTnV6Q?list=RDERCkf0j8pPM',
			image: 'https://img.youtube.com/vi/PAy5EBTnV6Q/hqdefault.jpg'
		});
		scene.flush();
		const editor = {
			editable: true,
			camera: { x: 0, y: 0, z: 1 },
			scene,
			selection: [],
			select: vi.fn(),
			editing: null,
			playing: null as string | null,
			finishEditing: vi.fn(),
			overlay: { hover: null },
			invalidateOverlay: vi.fn()
		};
		new SelectTool(editor as unknown as Editor).down({
			world: { x: 260, y: 160 },
			screen: { x: 260, y: 160 },
			pressure: 1,
			pointerType: 'mouse',
			shift: false,
			alt: false,
			meta: false,
			detail: 1,
			coalesced: []
		});
		expect(editor.playing).toBe(id);
		expect(open).not.toHaveBeenCalled();
	});
});
