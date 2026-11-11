import { compareZ, keysBetween } from '$lib/board/fractional';
import { isShapeType, LIMITS, type Binding, type Shape } from '$lib/board/schema';
import { pathPoints } from './bounds';
import type { Point } from './camera';
import type { Editor } from './editor.svelte';
import { center, rotatedBox, union } from './geometry';
import { movingIds } from './tools/move';

const MARK = 'toile/shapes';

function frozen(editor: Editor, shape: Shape): Shape {
	if (shape.type !== 'arrow' && shape.type !== 'line') return shape;
	const points = pathPoints(shape, editor.scene.get).flatMap((p) => [p.x - shape.x, p.y - shape.y]);
	return { ...shape, points, rot: 0 };
}

/** The clipboard text of the selection and the contents of its frames, back to front. */
export function serialize(editor: Editor): string | null {
	const ids = new Set([...editor.selection, ...movingIds(editor, editor.selection)]);
	const shapes = [...ids]
		.map((id) => editor.scene.get(id))
		.filter((s): s is Shape => !!s)
		.map((s) => frozen(editor, s));
	if (!shapes.length) return null;
	return JSON.stringify({ type: MARK, version: 1, shapes: shapes.sort(compareZ) });
}

/** The shapes in clipboard text, or null when it is not Toile's. */
export function parseClip(text: string): Shape[] | null {
	try {
		const data = JSON.parse(text) as { type?: string; shapes?: Shape[] };
		if (data.type !== MARK || !Array.isArray(data.shapes)) return null;
		return data.shapes.filter((s) => isShapeType(s.type) && Number.isFinite(s.x) && Number.isFinite(s.y));
	} catch {
		return null;
	}
}

function remap(binding: Binding | undefined, ids: Map<string, string>): Binding | undefined {
	return binding && ids.has(binding.shape)
		? { shape: ids.get(binding.shape) as string, anchor: binding.anchor }
		: undefined;
}

/**
 * Adds copies of `shapes` on top of the board, in one transaction (one undo step). Parents and arrow bindings are
 * remapped inside the copy and dropped when they point outside it. `at` centres the copy; otherwise it lands 24
 * units down and right of the originals. Selects and returns the new ids.
 */
export function insertShapes(editor: Editor, shapes: Shape[], at?: Point): string[] {
	if (!editor.editable || !shapes.length) return [];
	const room = LIMITS.shapes - editor.doc.shapes.size;
	const chosen = shapes.slice(0, Math.max(0, room));
	const box = union(chosen.map((s) => rotatedBox(s)));
	if (!box) return [];
	const middle = center(box);
	const [dx, dy] = at ? [at.x - middle.x, at.y - middle.y] : [24, 24];
	const order = editor.scene.order;
	const keys = keysBetween(order[order.length - 1]?.z ?? null, null, chosen.length);
	const ids = new Map<string, string>();
	const created: string[] = [];
	editor.doc.transact(() => {
		chosen.forEach((shape, i) => {
			const { id, type, text, ...fields } = shape;
			const copy = {
				...fields,
				x: shape.x + dx,
				y: shape.y + dy,
				z: keys[i],
				parent: undefined,
				from: undefined,
				to: undefined
			};
			const newId = editor.doc.create(type, copy, text ?? '');
			ids.set(id, newId);
			created.push(newId);
		});
		chosen.forEach((shape) => {
			const id = ids.get(shape.id) as string;
			const parent = shape.parent && ids.get(shape.parent);
			editor.doc.update(id, { parent: parent || undefined, from: remap(shape.from, ids), to: remap(shape.to, ids) });
		});
	});
	editor.select(created);
	return created;
}

/** Copies the selection in place, 24 units down and right. */
export function duplicate(editor: Editor): void {
	const text = serialize(editor);
	const shapes = text && parseClip(text);
	if (shapes) insertShapes(editor, shapes);
}
