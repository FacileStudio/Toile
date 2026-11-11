import { describe, expect, it } from 'vitest';
import * as Y from 'yjs';
import { BoardDoc, LOCAL } from './doc';
import { setText, splice } from './text';
import { createUndo } from './undo';

function peers(): [BoardDoc, BoardDoc] {
	const a = new BoardDoc();
	const b = new BoardDoc();
	a.ydoc.on(
		'update',
		(update: Uint8Array, origin: unknown) => origin !== 'peer' && Y.applyUpdate(b.ydoc, update, 'peer')
	);
	b.ydoc.on(
		'update',
		(update: Uint8Array, origin: unknown) => origin !== 'peer' && Y.applyUpdate(a.ydoc, update, 'peer')
	);
	return [a, b];
}

describe('BoardDoc', () => {
	it('creates a sticky with a Y.Text and reads it back as a plain shape', () => {
		const doc = new BoardDoc();
		doc.ensureMeta();
		const id = doc.create('sticky', { x: 10, y: 20, w: 200, h: 200, z: 'V', style: { fill: '#EEB47E' } }, 'Bonjour');
		expect(doc.meta.get('schema')).toBe(1);
		expect(doc.text(id)).toBeInstanceOf(Y.Text);
		expect(doc.get(id)).toEqual({
			id,
			type: 'sticky',
			x: 10,
			y: 20,
			w: 200,
			h: 200,
			rot: 0,
			z: 'V',
			style: { fill: '#EEB47E' },
			text: 'Bonjour'
		});
	});

	it('drops keys the type does not allow and clamps coordinates', () => {
		const doc = new BoardDoc();
		const id = doc.create('rect', { x: Infinity, y: 5e9, w: 10, h: 10, z: 'V', points: [1, 2], name: 'no' });
		const map = doc.shapes.get(id) as Y.Map<unknown>;
		expect(map.has('points')).toBe(false);
		expect(map.has('name')).toBe(false);
		expect(doc.get(id)?.x).toBe(0);
		expect(doc.get(id)?.y).toBe(1e7);
	});

	it('updates and deletes keys, and unbinds arrows and children of removed shapes', () => {
		const doc = new BoardDoc();
		const frame = doc.create('frame', { x: 0, y: 0, w: 100, h: 100, z: 'A', name: 'F' });
		const box = doc.create('rect', { x: 10, y: 10, w: 10, h: 10, z: 'B', parent: frame });
		const arrow = doc.create('arrow', {
			x: 0,
			y: 0,
			w: 1,
			h: 1,
			z: 'C',
			points: [0, 0, 1, 1],
			from: { shape: box, anchor: [1, 0.5] }
		});
		doc.update(box, { locked: true, parent: undefined });
		expect(doc.get(box)?.locked).toBe(true);
		expect(doc.get(box)?.parent).toBeUndefined();
		doc.update(box, { parent: frame });
		doc.remove([frame]);
		expect(doc.get(box)?.parent).toBeUndefined();
		doc.remove([box]);
		expect(doc.get(arrow)?.from).toBeUndefined();
	});

	it('never writes null or undefined, and keeps only valid style and binding values', () => {
		const doc = new BoardDoc();
		const style = { fill: '#EEB47E', stroke: null, width: 'x', align: 'middle', bogus: 1 } as unknown as Record<
			string,
			string
		>;
		const id = doc.create('arrow', {
			x: 0,
			y: 0,
			z: 'V',
			style,
			head: undefined,
			from: { shape: 'a', anchor: [2, -1] }
		});
		const map = doc.shapes.get(id) as Y.Map<unknown>;
		expect(map.get('style')).toEqual({ fill: '#EEB47E' });
		expect(map.has('head')).toBe(false);
		expect(map.get('from')).toEqual({ shape: 'a', anchor: [1, 0] });
		doc.update(id, { from: { shape: 1 } as unknown as undefined, locked: false });
		expect(map.has('from')).toBe(false);
		expect(map.has('locked')).toBe(false);
		const values = [...map.values()];
		expect(values.some((value) => value === null || value === undefined)).toBe(false);
	});

	it('ignores map entries that are not schema 1 shapes', () => {
		const doc = new BoardDoc();
		const bogus = new Y.Map();
		bogus.set('type', 'blob');
		doc.shapes.set('x', bogus);
		expect(doc.all()).toEqual([]);
	});

	it('edits text with one splice, so concurrent typing elsewhere survives', () => {
		expect(splice('hello world', 'hello brave world')).toEqual([6, 0, 'brave ']);
		const a = new BoardDoc();
		const b = new BoardDoc();
		const id = a.create('text', { x: 0, y: 0, w: 10, h: 10, z: 'V' }, 'abc def');
		Y.applyUpdate(b.ydoc, Y.encodeStateAsUpdate(a.ydoc));
		a.transact(() => setText(a.text(id) as Y.Text, 'abc! def'));
		b.transact(() => setText(b.text(id) as Y.Text, 'abc def?'));
		Y.applyUpdate(b.ydoc, Y.encodeStateAsUpdate(a.ydoc));
		Y.applyUpdate(a.ydoc, Y.encodeStateAsUpdate(b.ydoc));
		expect(a.get(id)?.text).toBe('abc! def?');
		expect(b.get(id)?.text).toBe('abc! def?');
	});
});

describe('undo', () => {
	it('undoes only local writes, never a peer edit', () => {
		const [a, b] = peers();
		const undo = createUndo(a);
		const mine = a.create('rect', { x: 0, y: 0, w: 10, h: 10, z: 'V' });
		undo.stopCapturing();
		const theirs = b.create('ellipse', { x: 5, y: 5, w: 10, h: 10, z: 'W' });
		a.update(mine, { x: 50 });
		undo.undo();
		expect(a.get(mine)?.x).toBe(0);
		expect(a.get(theirs)).not.toBeNull();
		undo.undo();
		expect(a.get(mine)).toBeNull();
		expect(b.get(theirs)).not.toBeNull();
		undo.redo();
		expect(b.get(mine)).not.toBeNull();
	});

	it('tags local writes with the LOCAL origin', () => {
		const doc = new BoardDoc();
		const origins: unknown[] = [];
		doc.ydoc.on('afterTransaction', (tr: Y.Transaction) => origins.push(tr.origin));
		doc.create('rect', { x: 0, y: 0, w: 1, h: 1, z: 'V' });
		expect(origins).toEqual([LOCAL]);
	});
});
