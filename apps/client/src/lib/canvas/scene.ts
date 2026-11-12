import type * as Y from 'yjs';
import { prefersReducedMotion } from '@facile/muse';
import { readShape, type BoardDoc } from '$lib/board/doc';
import { compareZ } from '$lib/board/fractional';
import { PATH_TYPES, type Shape } from '$lib/board/schema';
import { shapeBounds } from './bounds';
import type { Box } from './camera';
import { intersects } from './geometry';
import { SpatialIndex } from './spatial';

/** How long a new shape pops in, in milliseconds. */
export const POP_MS = 320;

/** How long a removed shape fades out, in milliseconds. */
export const FADE_MS = 200;

const MOTION_LIMIT = 200;

const still = () => typeof matchMedia !== 'function' || prefersReducedMotion();

type Motion = { born: string[]; gone: Shape[] };

/**
 * The renderer's read model of the board: plain shape snapshots, their z order, a spatial index and the arrows
 * bound to each shape. Document changes queue ids; `flush` applies them once per animation frame, so a burst of
 * remote updates costs one pass.
 */
export class Scene {
	readonly shapes = new Map<string, Shape>();
	readonly index = new SpatialIndex();
	private ordered: Shape[] = [];
	private ranks = new Map<string, number>();
	private orderDirty = true;
	private pending = new Set<string>();
	private arrows = new Map<string, Set<string>>();
	private unobserve: () => void;
	/** Whether changes animate: false until the first sync has landed, so opening a board does not pop it in. */
	live = false;
	version = 0;
	/** When each shape that just appeared was added, for its pop-in. */
	readonly births = new Map<string, number>();
	/** Shapes that just disappeared, kept for their fade-out, with when they went. */
	readonly ghosts = new Map<string, { shape: Shape; at: number }>();

	constructor(
		readonly doc: BoardDoc,
		private onchange: () => void
	) {
		const observer = (events: Y.YEvent<Y.AbstractType<unknown>>[]) => this.queue(events);
		doc.shapes.observeDeep(observer);
		this.unobserve = () => doc.shapes.unobserveDeep(observer);
		doc.shapes.forEach((_map, id) => this.pending.add(id));
		this.flush();
	}

	/** Stops listening to the document. */
	destroy(): void {
		this.unobserve();
	}

	/** Whether changes wait for the next `flush`. */
	get dirty(): boolean {
		return this.pending.size > 0;
	}

	/** Every shape, back to front. */
	get order(): Shape[] {
		if (this.orderDirty) this.sort();
		return this.ordered;
	}

	/** The stacking rank of shape `id` (higher draws later). */
	rank(id: string): number {
		if (this.orderDirty) this.sort();
		return this.ranks.get(id) ?? -1;
	}

	/** Looks a shape up by id. */
	readonly get = (id: string): Shape | undefined => this.shapes.get(id);

	/** The ids bound as ends of arrows to shape `id`. */
	arrowsOf(id: string): ReadonlySet<string> {
		return this.arrows.get(id) ?? new Set();
	}

	/** The shapes whose bounds overlap `area`, back to front. */
	within(area: Box): Shape[] {
		if (this.index.broad(area)) {
			return this.order.filter((shape) => {
				const box = this.index.box(shape.id);
				return !!box && intersects(box, area);
			});
		}
		const hits = this.index.query(area);
		if (hits.size > this.shapes.size / 4) return this.order.filter((shape) => hits.has(shape.id));
		return [...hits].map((id) => this.shapes.get(id) as Shape).sort((a, b) => this.rank(a.id) - this.rank(b.id));
	}

	/** Applies every queued change. Returns whether anything changed. */
	flush(): boolean {
		if (!this.pending.size) return false;
		const ids = this.pending;
		this.pending = new Set();
		const moved = new Set<string>();
		const motion: Motion = { born: [], gone: [] };
		for (const id of ids) this.apply(id, moved, motion);
		for (const id of moved) for (const arrow of this.arrowsOf(id)) this.reindex(arrow);
		this.animate(motion);
		this.version += 1;
		return true;
	}

	/** Drops finished pop-ins and fade-outs. Returns whether any are still running at `now`. */
	settle(now: number): boolean {
		for (const [id, at] of this.births) if (now - at >= POP_MS) this.births.delete(id);
		for (const [id, ghost] of this.ghosts) if (now - ghost.at >= FADE_MS) this.ghosts.delete(id);
		return this.births.size > 0 || this.ghosts.size > 0;
	}

	private animate(motion: Motion): void {
		if (!this.live || still()) return;
		const now = performance.now();
		if (motion.born.length <= MOTION_LIMIT)
			for (const id of motion.born) {
				this.ghosts.delete(id);
				this.births.set(id, now);
			}
		if (motion.gone.length <= MOTION_LIMIT) for (const shape of motion.gone) this.ghosts.set(shape.id, { shape, at: now });
	}

	private queue(events: Y.YEvent<Y.AbstractType<unknown>>[]): void {
		for (const event of events) {
			if (event.path.length) this.pending.add(String(event.path[0]));
			else event.changes.keys.forEach((_change, key) => this.pending.add(key));
		}
		this.onchange();
	}

	private apply(id: string, moved: Set<string>, motion: Motion): void {
		const before = this.shapes.get(id);
		if (before) this.unbind(before);
		const map = this.doc.shapes.get(id);
		const shape = map ? readShape(id, map) : null;
		if (!before && shape && !PATH_TYPES.has(shape.type)) motion.born.push(id);
		if (before && !shape && before.type !== 'frame') motion.gone.push(before);
		if (!shape) {
			this.shapes.delete(id);
			this.index.delete(id);
		} else {
			this.shapes.set(id, shape);
			this.bind(shape);
			this.index.set(id, shapeBounds(shape, this.get));
		}
		if (!before || !shape || before.z !== shape.z) this.orderDirty = true;
		else if (!this.orderDirty) this.ordered[this.ranks.get(id) as number] = shape;
		moved.add(id);
	}

	private reindex(id: string): void {
		const shape = this.shapes.get(id);
		if (shape) this.index.set(id, shapeBounds(shape, this.get));
	}

	private bind(shape: Shape): void {
		for (const end of [shape.from, shape.to]) {
			if (!end) continue;
			let set = this.arrows.get(end.shape);
			if (!set) this.arrows.set(end.shape, (set = new Set()));
			set.add(shape.id);
		}
	}

	private unbind(shape: Shape): void {
		for (const end of [shape.from, shape.to]) if (end) this.arrows.get(end.shape)?.delete(shape.id);
	}

	private sort(): void {
		this.ordered = [...this.shapes.values()].sort(compareZ);
		this.ranks = new Map(this.ordered.map((shape, i) => [shape.id, i]));
		this.orderDirty = false;
	}
}
