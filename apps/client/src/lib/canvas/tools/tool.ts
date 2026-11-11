import type { Point } from '../camera';

/** The tools of the toolbar. `image` opens the file picker; the rest drive pointer gestures. */
export type ToolName =
	| 'select'
	| 'hand'
	| 'sticky'
	| 'text'
	| 'rect'
	| 'ellipse'
	| 'diamond'
	| 'line'
	| 'arrow'
	| 'pen'
	| 'eraser'
	| 'frame'
	| 'comment';

/** One pointer sample, in both spaces, with what the pen and the keyboard say. */
export type PointerSample = {
	world: Point;
	screen: Point;
	pressure: number;
	pointerType: string;
	shift: boolean;
	alt: boolean;
	meta: boolean;
	detail: number;
	coalesced: Point[];
};

/** A pointer gesture handler: one per tool. A gesture runs from `down` to `up` or `cancel`. */
export interface Tool {
	readonly cursor: string;
	down(sample: PointerSample): void;
	move(sample: PointerSample): void;
	up(sample: PointerSample): void;
	cancel(): void;
	hover?(sample: PointerSample): void;
}

/** The tools that create a box shape by click or drag. */
export const BOX_TOOLS = new Set<ToolName>(['sticky', 'text', 'rect', 'ellipse', 'diamond', 'frame']);

/** The tools a role that cannot edit may still use. */
export const VIEW_TOOLS = new Set<ToolName>(['select', 'hand']);
