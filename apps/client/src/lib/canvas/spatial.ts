import type { Box } from './camera';
import { intersects } from './geometry';

const CELL = 512;
const MAX_CELLS = 64;

function cellKey(cx: number, cy: number): number {
	return ((cx * 73856093) ^ (cy * 19349663)) | 0;
}

/**
 * A uniform-grid spatial hash over world boxes. A box covering more than `MAX_CELLS` cells goes to a short list
 * every query scans, so one huge frame cannot fill thousands of cells.
 */
export class SpatialIndex {
	private cells = new Map<number, Set<string>>();
	private boxes = new Map<string, Box>();
	private keys = new Map<string, number[]>();
	private large = new Set<string>();

	/** How many ids the index holds. */
	get size(): number {
		return this.boxes.size;
	}

	/** The box stored for `id`. */
	box(id: string): Box | undefined {
		return this.boxes.get(id);
	}

	/** Inserts or moves `id` to `box`. */
	set(id: string, box: Box): void {
		this.delete(id);
		this.boxes.set(id, box);
		const span = cellSpan(box);
		if ((span.x1 - span.x0 + 1) * (span.y1 - span.y0 + 1) > MAX_CELLS) {
			this.large.add(id);
			return;
		}
		const keys: number[] = [];
		for (let cx = span.x0; cx <= span.x1; cx += 1) {
			for (let cy = span.y0; cy <= span.y1; cy += 1) keys.push(cellKey(cx, cy));
		}
		for (const key of keys) {
			let cell = this.cells.get(key);
			if (!cell) this.cells.set(key, (cell = new Set()));
			cell.add(id);
		}
		this.keys.set(id, keys);
	}

	/** Removes `id`. */
	delete(id: string): void {
		this.boxes.delete(id);
		this.large.delete(id);
		for (const key of this.keys.get(id) ?? []) {
			const cell = this.cells.get(key);
			cell?.delete(id);
			if (cell && cell.size === 0) this.cells.delete(key);
		}
		this.keys.delete(id);
	}

	/** Whether `area` spans more grid cells than are occupied, so scanning every box beats walking cells. */
	broad(area: Box): boolean {
		const span = cellSpan(area);
		return (span.x1 - span.x0 + 1) * (span.y1 - span.y0 + 1) > this.cells.size;
	}

	/** Every id whose box overlaps `area`. */
	query(area: Box): Set<string> {
		const out = new Set<string>();
		const span = cellSpan(area);
		const cellCount = (span.x1 - span.x0 + 1) * (span.y1 - span.y0 + 1);
		if (cellCount > this.cells.size) return this.scan(area);
		for (let cx = span.x0; cx <= span.x1; cx += 1) {
			for (let cy = span.y0; cy <= span.y1; cy += 1) this.collect(cellKey(cx, cy), area, out);
		}
		for (const id of this.large) if (intersects(this.boxes.get(id) as Box, area)) out.add(id);
		return out;
	}

	private collect(key: number, area: Box, out: Set<string>): void {
		for (const id of this.cells.get(key) ?? []) {
			if (!out.has(id) && intersects(this.boxes.get(id) as Box, area)) out.add(id);
		}
	}

	private scan(area: Box): Set<string> {
		const out = new Set<string>();
		for (const [id, box] of this.boxes) if (intersects(box, area)) out.add(id);
		return out;
	}
}

function cellSpan(box: Box): { x0: number; y0: number; x1: number; y1: number } {
	return {
		x0: Math.floor(box.x / CELL),
		y0: Math.floor(box.y / CELL),
		x1: Math.floor((box.x + box.w) / CELL),
		y1: Math.floor((box.y + box.h) / CELL)
	};
}
