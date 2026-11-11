import { LIMITS } from '$lib/board/schema';
import { createShape } from '../actions';
import { pointsBox } from '../bounds';
import type { Editor } from '../editor.svelte';
import { frameAt } from '../hit';
import type { PointerSample, Tool } from './tool';

/** The pressure perfect-freehand reads as "no pressure data": it then simulates pressure from speed. */
const FLAT = 0.5;

/** Draws freehand ink with pressure from a pen, or simulated from speed for a mouse or a finger. */
export class PenTool implements Tool {
	cursor = 'crosshair';
	private points: number[] | null = null;

	constructor(private editor: Editor) {}

	private width(): number {
		return this.editor.style.width ?? 4;
	}

	down(sample: PointerSample): void {
		this.points = [];
		this.add(sample, [sample.world]);
	}

	move(sample: PointerSample): void {
		if (!this.points) return;
		this.add(sample, sample.coalesced.length ? sample.coalesced.map((p) => this.editor.world(p)) : [sample.world]);
		if (this.points.length / 3 >= LIMITS.inkPoints) {
			this.commit();
			this.points = [];
			this.add(sample, [sample.world]);
		}
	}

	up(): void {
		this.commit();
		this.cancel();
	}

	cancel(): void {
		this.points = null;
		this.editor.overlay.draft = null;
		this.editor.invalidateOverlay();
	}

	private add(sample: PointerSample, worlds: { x: number; y: number }[]): void {
		const pressure =
			sample.pointerType === 'pen' && sample.pressure > 0 ? Math.round(sample.pressure * 100) / 100 : FLAT;
		for (const p of worlds) this.points?.push(Math.round(p.x * 100) / 100, Math.round(p.y * 100) / 100, pressure);
		const color = this.editor.style.stroke ?? this.editor.palette.fg;
		this.editor.overlay.draft = { kind: 'ink', points: this.points ?? [], width: this.width(), color };
		this.editor.invalidateOverlay();
	}

	private commit(): void {
		const points = this.points;
		if (!points || points.length < 3) return;
		const box = pointsBox(points, 3);
		const local = points.map((value, i) => (i % 3 === 0 ? value - box.x : i % 3 === 1 ? value - box.y : value));
		const parent = frameAt(this.editor.scene, { x: box.x + box.w / 2, y: box.y + box.h / 2 }, new Set());
		createShape(this.editor, 'ink', {
			x: box.x,
			y: box.y,
			w: box.w,
			h: box.h,
			points: local,
			parent: parent?.id,
			style: { ...(this.editor.style.stroke ? { stroke: this.editor.style.stroke } : {}), width: this.width() }
		});
	}
}
