import { afterEach, describe, expect, it, vi } from 'vitest';
import { BoardDoc } from '$lib/board/doc';
import { imageLoader } from '$lib/board/assets';
import { openShape, SelectTool } from './tools/select';
import { Scene } from './scene';
import { pasteInto } from './paste';
import type { Editor } from './editor.svelte';
import type { Shape } from '$lib/board/schema';

afterEach(() => vi.unstubAllGlobals());

describe('card shape and tools', () => {
	it('preserves card metadata in doc', () => {
		const doc = new BoardDoc();
		const id = doc.create('card', {
			x: 0,
			y: 0,
			w: 320,
			h: 180,
			z: 'V',
			url: 'https://example.com',
			title: 'Example',
			kind: 'link',
			image: 'https://example.com/banner.png',
			description: 'A test page',
			favicon: 'https://example.com/favicon.ico',
			site: 'example.com'
		});
		const shape = doc.get(id);
		expect(shape?.image).toBe('https://example.com/banner.png');
		expect(shape?.description).toBe('A test page');
		expect(shape?.favicon).toBe('https://example.com/favicon.ico');
		expect(shape?.site).toBe('example.com');
	});

	it('opens card url in new tab on openShape', () => {
		const open = vi.fn();
		vi.stubGlobal('open', open);
		const shape: Shape = {
			id: 'c1',
			type: 'card',
			x: 0,
			y: 0,
			w: 320,
			h: 100,
			rot: 0,
			z: 'V',
			style: {},
			url: 'https://example.com/test'
		};
		openShape({ editable: true } as unknown as Editor, shape);
		expect(open).toHaveBeenCalledWith('https://example.com/test', '_blank', 'noopener');
	});

	it('loads direct http and data urls in imageLoader and calls onload for cached images', () => {
		const onload = vi.fn();
		const loader = imageLoader('board1', onload);
		const FakeImage = class {
			onload: (() => void) | null = null;
			complete = true;
			naturalWidth = 100;
			src = '';
		};
		vi.stubGlobal('Image', FakeImage);
		const res = loader('https://example.com/img.png');
		expect(res).not.toBeNull();
		expect(onload).toHaveBeenCalled();
	});

	it('creates an image shape when pasting an image url', async () => {
		const doc = new BoardDoc();
		const editor = {
			editable: true,
			world: (p: { x: number; y: number }) => p,
			size: { width: 800, height: 600 },
			scene: { shapes: doc.shapes, order: [] },
			doc,
			select: vi.fn(),
			host: { uploadImage: vi.fn() }
		};
		const data = {
			files: [],
			getData: (t: string) => (t === 'text/plain' ? 'https://example.com/photo.png' : '')
		};
		await pasteInto(editor as unknown as Editor, data as unknown as DataTransfer, { x: 100, y: 100 });
		const shapes = doc.all();
		expect(shapes).toHaveLength(1);
		expect(shapes[0].type).toBe('image');
		expect(shapes[0].asset).toBe('https://example.com/photo.png');
		expect(shapes[0].w).toBe(320);
		expect(shapes[0].h).toBe(240);
	});

	it('creates a youtube card when pasting a youtube link', async () => {
		const doc = new BoardDoc();
		const editor = {
			editable: true,
			world: (p: { x: number; y: number }) => p,
			size: { width: 800, height: 600 },
			scene: { shapes: doc.shapes, order: [] },
			doc,
			select: vi.fn(),
			host: { uploadImage: vi.fn() }
		};
		const data = {
			files: [],
			getData: (t: string) => (t === 'text/plain' ? 'https://www.youtube.com/watch?v=dQw4w9WgXcQ' : '')
		};
		await pasteInto(editor as unknown as Editor, data as unknown as DataTransfer, { x: 100, y: 100 });
		const shapes = doc.all();
		expect(shapes).toHaveLength(1);
		expect(shapes[0].type).toBe('card');
		expect(shapes[0].kind).toBe('youtube');
		expect(shapes[0].title).toBe('YouTube Video');
		expect(shapes[0].w).toBe(320);
		expect(shapes[0].h).toBe(200);
	});

	it('opens card url directly when clicking play button or external link icon', () => {
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
			url: 'https://www.youtube.com/watch?v=123',
			image: 'https://example.com/thumb.jpg'
		});
		scene.flush();
		const select = vi.fn();
		const editor = {
			editable: true,
			camera: { x: 0, y: 0, z: 1 },
			scene,
			selection: [],
			select,
			editing: null,
			finishEditing: vi.fn(),
			overlay: { hover: null },
			invalidateOverlay: vi.fn()
		};
		const tool = new SelectTool(editor as unknown as Editor);
		tool.down({
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
		expect(open).toHaveBeenCalledWith('https://www.youtube.com/watch?v=123', '_blank', 'noopener');
		expect(select).toHaveBeenCalledWith([id]);
	});

	it('opens card url directly when clicking external link icon', () => {
		const open = vi.fn();
		vi.stubGlobal('open', open);
		const doc = new BoardDoc();
		const scene = new Scene(doc, () => undefined);
		const id = doc.create('card', {
			x: 100,
			y: 100,
			w: 320,
			h: 100,
			z: 'V',
			kind: 'link',
			url: 'https://example.com/site'
		});
		scene.flush();
		const select = vi.fn();
		const editor = {
			editable: true,
			camera: { x: 0, y: 0, z: 1 },
			scene,
			selection: [],
			select,
			editing: null,
			finishEditing: vi.fn(),
			overlay: { hover: null },
			invalidateOverlay: vi.fn()
		};
		const tool = new SelectTool(editor as unknown as Editor);
		tool.down({
			world: { x: 398, y: 174 },
			screen: { x: 398, y: 174 },
			pressure: 1,
			pointerType: 'mouse',
			shift: false,
			alt: false,
			meta: false,
			detail: 1,
			coalesced: []
		});
		expect(open).toHaveBeenCalledWith('https://example.com/site', '_blank', 'noopener');
		expect(select).toHaveBeenCalledWith([id]);
	});

	it('changes cursor to pointer when hovering over external link icon or youtube play button', () => {
		const doc = new BoardDoc();
		const scene = new Scene(doc, () => undefined);
		doc.create('card', {
			x: 100,
			y: 100,
			w: 320,
			h: 100,
			z: 'V',
			kind: 'link',
			url: 'https://example.com/site'
		});
		scene.flush();
		const editor = {
			editable: true,
			camera: { x: 0, y: 0, z: 1 },
			scene,
			selection: [],
			select: vi.fn(),
			editing: null,
			finishEditing: vi.fn(),
			overlay: { hover: null },
			invalidateOverlay: vi.fn()
		};
		const tool = new SelectTool(editor as unknown as Editor);
		expect(tool.cursor).toBe('default');
		tool.hover?.({
			world: { x: 398, y: 174 },
			screen: { x: 398, y: 174 },
			pressure: 0,
			pointerType: 'mouse',
			shift: false,
			alt: false,
			meta: false,
			detail: 0,
			coalesced: []
		});
		expect(tool.cursor).toBe('pointer');
		tool.hover?.({
			world: { x: 120, y: 120 },
			screen: { x: 120, y: 120 },
			pressure: 0,
			pointerType: 'mouse',
			shift: false,
			alt: false,
			meta: false,
			detail: 0,
			coalesced: []
		});
		expect(tool.cursor).toBe('default');
	});
});
