import type * as Y from 'yjs';
import { BoardDoc } from '../src/lib/board/doc';
import { keysBetween } from '../src/lib/board/fractional';
import type { ShapeType } from '../src/lib/board/schema';

const COLORS = ['#AD9EF0', '#F09ED6', '#EE7E89', '#EEB47E', '#A9EE7E', '#7EEEDB'];
const KINDS: ShapeType[] = [
	'sticky',
	'sticky',
	'sticky',
	'sticky',
	'rect',
	'rect',
	'ellipse',
	'diamond',
	'ink',
	'arrow'
];

function ink(i: number): number[] {
	const points: number[] = [];
	for (let k = 0; k < 30; k += 1) points.push(k * 6, 40 + Math.sin((k + i) / 4) * 30, 0.5);
	return points;
}

/** Fills `ydoc` with `count` shapes on a grid: stickies with text, boxes, ink and bound arrows. Returns the ids. */
export function seedBoard(ydoc: Y.Doc, count: number): string[] {
	const doc = new BoardDoc(ydoc);
	const keys = keysBetween(null, null, count);
	const ids: string[] = [];
	const columns = Math.ceil(Math.sqrt(count));
	doc.ensureMeta();
	doc.transact(() => {
		for (let i = 0; i < count; i += 1) {
			const x = (i % columns) * 260;
			const y = Math.floor(i / columns) * 260;
			const type = KINDS[i % KINDS.length];
			const style = type === 'sticky' ? { fill: COLORS[i % COLORS.length] } : {};
			const fields = { x, y, w: 200, h: type === 'ink' ? 80 : 200, z: keys[i], style };
			if (type === 'ink') ids.push(doc.create('ink', { ...fields, points: ink(i), style: {} }));
			else if (type === 'arrow' && ids.length > 1) {
				const from = { shape: ids[ids.length - 2], anchor: [1, 0.5] as [number, number] };
				const to = { shape: ids[ids.length - 1], anchor: [0, 0.5] as [number, number] };
				ids.push(doc.create('arrow', { ...fields, points: [0, 0, 200, 0], from, to, head: 'end', style: {} }));
			} else
				ids.push(doc.create(type === 'arrow' ? 'rect' : type, fields, type === 'sticky' ? `Idée numéro ${i}` : ''));
		}
	});
	return ids;
}
