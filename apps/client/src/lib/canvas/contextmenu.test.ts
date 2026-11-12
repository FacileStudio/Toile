import { describe, expect, it, vi } from 'vitest';
import { BoardDoc } from '$lib/board/doc';
import { createShape, deleteSelection } from './actions';
import { duplicate, insertShapes, parseClip, serialize } from './clipboard';
import { pasteInto } from './paste';
import { canvasMenuItems, elementMenuItems } from './contextmenu';
import type { Editor } from './editor.svelte';
import type { Shape } from '$lib/board/schema';

describe('canvas and element context menu operations', () => {
	it('creates shapes at target world coordinates', () => {
		const doc = new BoardDoc();
		const editor = {
			editable: true,
			scene: { order: [], shapes: doc.shapes },
			doc,
			style: {}
		} as unknown as Editor;

		const stickyId = createShape(editor, 'sticky', { x: 150, y: 250, w: 200, h: 200 });
		expect(stickyId).not.toBeNull();
		const sticky = doc.get(stickyId!);
		expect(sticky?.type).toBe('sticky');
		expect(sticky?.x).toBe(150);
		expect(sticky?.y).toBe(250);

		const textId = createShape(editor, 'text', { x: 300, y: 400, w: 100, h: 32 });
		expect(textId).not.toBeNull();
		const text = doc.get(textId!);
		expect(text?.type).toBe('text');
		expect(text?.x).toBe(300);
		expect(text?.y).toBe(400);
	});

	it('serializes and duplicates selection', () => {
		const doc = new BoardDoc();
		const id = doc.create('rect', { x: 50, y: 60, w: 100, h: 100, z: 'V', style: {} });
		const shape = doc.get(id)!;
		const editor = {
			editable: true,
			selection: [id],
			scene: {
				get: (sId: string) => (sId === id ? shape : null),
				shapes: doc.shapes,
				order: [shape],
				arrowsOf: () => []
			},
			doc,
			select: vi.fn()
		} as unknown as Editor;

		const clip = serialize(editor);
		expect(clip).not.toBeNull();
		const parsed = parseClip(clip!);
		expect(parsed).toHaveLength(1);
		expect(parsed![0].type).toBe('rect');

		duplicate(editor);
		expect(doc.shapes.size).toBe(2);
	});

	it('deletes selection on delete or cut', () => {
		const doc = new BoardDoc();
		const id = doc.create('sticky', { x: 0, y: 0, w: 100, h: 100, z: 'V', style: {} });
		const shape = doc.get(id)!;
		let selection = [id];
		const editor = {
			editable: true,
			get selection() {
				return selection;
			},
			set selection(s) {
				selection = s;
			},
			scene: {
				get: (sId: string) => (sId === id ? shape : null),
				shapes: doc.shapes,
				order: [shape],
				arrowsOf: () => []
			},
			doc,
			select: vi.fn((s) => {
				selection = s;
			})
		} as unknown as Editor;

		deleteSelection(editor);
		expect(doc.shapes.size).toBe(0);
		expect(selection).toEqual([]);
	});

	it('pastes serialized shapes at right-click world location', async () => {
		const doc = new BoardDoc();
		const orig: Shape = {
			id: 'r1',
			type: 'rect',
			x: 0,
			y: 0,
			w: 100,
			h: 100,
			rot: 0,
			z: 'V',
			style: {}
		};
		const clipText = JSON.stringify({
			type: 'toile/shapes',
			version: 1,
			shapes: [orig]
		});
		const editor = {
			editable: true,
			scene: { shapes: doc.shapes, order: [] },
			doc,
			select: vi.fn()
		} as unknown as Editor;

		const data = {
			files: [],
			getData: (format: string) => (format === 'text/plain' ? clipText : '')
		};

		await pasteInto(editor, data as unknown as DataTransfer, { x: 500, y: 600 });
		const all = doc.all();
		expect(all).toHaveLength(1);
		expect(all[0].type).toBe('rect');
		expect(all[0].x).toBe(450);
		expect(all[0].y).toBe(550);
	});

	it('builds canvas menu items with quick creation and clipboard actions', () => {
		const editor = { editable: true } as unknown as Editor;
		const ui = { help: vi.fn(), search: vi.fn(), image: vi.fn(), minimap: vi.fn() };
		const items = canvasMenuItems(editor, ui, { x: 100, y: 200 });
		const labels = items.map((i) => i.label);
		expect(labels).toContain('Coller');
		expect(labels).toContain('Ajouter une note');
		expect(labels).toContain('Ajouter un texte');
		expect(labels).toContain('Ajouter un rectangle');
		expect(labels).toContain('Ajouter un cadre');
		expect(labels).toContain('Tout sélectionner');
		expect(labels).toContain('Tout afficher');
		expect(labels).toContain('Zoom à 100 %');
	});

	it('builds element menu items with smart options for cards', () => {
		const card: Shape = {
			id: 'c1',
			type: 'card',
			x: 0,
			y: 0,
			w: 320,
			h: 180,
			rot: 0,
			z: 'V',
			style: {},
			url: 'https://example.com'
		};
		const editor = {
			editable: true,
			selection: ['c1'],
			scene: {
				get: (id: string) => (id === 'c1' ? card : null)
			}
		} as unknown as Editor;
		const ui = { help: vi.fn(), search: vi.fn(), image: vi.fn(), minimap: vi.fn() };
		const items = elementMenuItems(editor, ui, card);
		const labels = items.map((i) => i.label);
		expect(labels).toContain('Ouvrir le lien');
		expect(labels).toContain('Copier le lien');
		expect(labels).toContain('Couper');
		expect(labels).toContain('Copier');
		expect(labels).toContain('Dupliquer');
		expect(labels).toContain('Supprimer');
		expect(labels).toContain('Premier plan');
		expect(labels).toContain('Arrière-plan');
		expect(labels).toContain('Verrouiller');
		expect(labels).toContain('Zoomer sur la sélection');
	});
});
