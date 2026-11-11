import * as Y from 'yjs';
import {
	LIMITS,
	SCHEMA,
	TEXT_TYPES,
	allowedKey,
	clampNumber,
	cleanBinding,
	cleanStyle,
	isShapeType,
	type Shape,
	type ShapeFields,
	type ShapeType
} from './schema';

/** The transaction origin of every write this client makes; the undo manager tracks only it. */
export const LOCAL = Symbol('toile.local');

const NUMERIC = new Set(['x', 'y', 'w', 'h', 'rot']);

/** A new shape id: 16 url-safe random characters. */
export function newId(): string {
	const bytes = crypto.getRandomValues(new Uint8Array(12));
	return btoa(String.fromCharCode(...bytes))
		.replace(/\+/g, '-')
		.replace(/\//g, '_');
}

function clean(key: string, value: unknown): unknown {
	if (NUMERIC.has(key)) return clampNumber(value as number);
	if (key === 'points') return Array.isArray(value) ? value.map((n) => clampNumber(Number(n))) : undefined;
	if (key === 'style') return cleanStyle(value);
	if (key === 'from' || key === 'to') return cleanBinding(value);
	if (key === 'locked') return value === true ? true : undefined;
	if (typeof value === 'string') return value.slice(0, 10_000);
	return value;
}

/** Reads one shape's Y.Map into a plain snapshot, or null when it is not a schema 1 shape. */
export function readShape(id: string, map: Y.Map<unknown>): Shape | null {
	const type = map.get('type');
	if (!isShapeType(type)) return null;
	const shape: Shape = { id, type, x: 0, y: 0, w: 0, h: 0, rot: 0, z: '', style: {} };
	for (const [key, value] of map.entries()) {
		if (value instanceof Y.Text) (shape as Record<string, unknown>)[key] = value.toString();
		else if (value !== undefined) (shape as Record<string, unknown>)[key] = value;
	}
	return shape;
}

function unlink(map: Y.Map<unknown>, gone: Set<string>): void {
	for (const key of ['from', 'to'] as const) {
		const binding = map.get(key) as { shape: string } | undefined;
		if (binding && gone.has(binding.shape)) map.delete(key);
	}
	if (gone.has(map.get('parent') as string)) map.delete('parent');
}

/** Typed accessors over one board's Y.Doc. Every write runs in a transaction with the `LOCAL` origin. */
export class BoardDoc {
	readonly ydoc: Y.Doc;
	readonly shapes: Y.Map<Y.Map<unknown>>;
	readonly meta: Y.Map<unknown>;

	constructor(ydoc: Y.Doc = new Y.Doc()) {
		this.ydoc = ydoc;
		this.shapes = ydoc.getMap('shapes');
		this.meta = ydoc.getMap('meta');
	}

	/** Stamps `meta.schema` when the synced document has none yet. */
	ensureMeta(): void {
		if (this.meta.get('schema') === undefined) this.transact(() => this.meta.set('schema', SCHEMA));
	}

	/** Runs `fn` in one local transaction, so it is one undo step and one network update. */
	transact(fn: () => void): void {
		this.ydoc.transact(fn, LOCAL);
	}

	/** The shape `id`, or null. */
	get(id: string): Shape | null {
		const map = this.shapes.get(id);
		return map ? readShape(id, map) : null;
	}

	/** Every schema 1 shape in the document, in no order. */
	all(): Shape[] {
		const out: Shape[] = [];
		this.shapes.forEach((map, id) => {
			const shape = readShape(id, map);
			if (shape) out.push(shape);
		});
		return out;
	}

	/** Creates a shape and returns its id. `text` seeds the Y.Text of the types that carry one. */
	create(type: ShapeType, fields: ShapeFields, text = ''): string {
		if (this.shapes.size >= LIMITS.shapes) throw new Error('board-full');
		const id = newId();
		const map = new Y.Map<unknown>();
		map.set('type', type);
		for (const [key, value] of Object.entries({ rot: 0, style: {}, ...fields })) {
			const kept = key === 'type' || !allowedKey(type, key) || value == null ? undefined : clean(key, value);
			if (kept !== undefined) map.set(key, kept);
		}
		if (TEXT_TYPES.has(type)) map.set('text', new Y.Text(text.slice(0, LIMITS.textChars)));
		this.transact(() => this.shapes.set(id, map));
		return id;
	}

	/** Sets the given fields on shape `id`; `undefined` deletes a key. Unknown keys are dropped. */
	update(id: string, fields: ShapeFields): void {
		const map = this.shapes.get(id);
		const type = map?.get('type');
		if (!map || !isShapeType(type)) return;
		this.transact(() => {
			for (const [key, value] of Object.entries(fields)) {
				if (key === 'type' || !allowedKey(type, key)) continue;
				const kept = value == null ? undefined : clean(key, value);
				if (kept === undefined) map.delete(key);
				else if (map.get(key) !== kept) map.set(key, kept);
			}
		});
	}

	/** Applies many updates in one transaction. */
	updateMany(changes: Map<string, ShapeFields>): void {
		this.transact(() => changes.forEach((fields, id) => this.update(id, fields)));
	}

	/** Deletes shapes, and the arrow bindings and frame parents that pointed at them. */
	remove(ids: Iterable<string>): void {
		const gone = new Set(ids);
		this.transact(() => {
			gone.forEach((id) => this.shapes.delete(id));
			this.shapes.forEach((map) => unlink(map, gone));
		});
	}

	/** The Y.Text of shape `id`, or null when its type carries none. */
	text(id: string): Y.Text | null {
		const value = this.shapes.get(id)?.get('text');
		return value instanceof Y.Text ? value : null;
	}
}
