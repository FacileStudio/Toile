import { USER_COLORS } from '@facile/muse';
import { cursorAt, type CursorState } from '@facile/mise-en-scene';
import { toolIcons } from '$lib/canvas/icons';
import { t } from '$lib/i18n/index.svelte';

/** A rectangle on the faux board, in the window body's design pixels. */
export type Box = { x: number; y: number; w: number; h: number };

/** A point in design pixels. */
export type Spot = { x: number; y: number };

/** A point inside a box, as fractions of its width and height. */
export type Anchor = [number, number];

/** Where everything sits in one design of the hero board. */
export type HeroLayout = {
	window: string;
	origin: Spot;
	frames: [Box, Box];
	onboarding: Box;
	export: Box;
	beta: Box;
	extra: Box[];
	drag: Spot;
	arrow: string;
	head: string;
	from: Anchor;
	to: Anchor;
	toolbar: 'column' | 'row';
};

/** The fractions of a sticky each collaborator grabs or clicks. */
export const GRIP = { lea: [0.55, 0.35], malik: [0.55, 0.62], note: [0.75, 0.85] } as const satisfies Record<string, Anchor>;

/** The 960 × 640 design under the hero copy. */
export const WIDE: HeroLayout = {
	window: 'top:16px;left:50px;width:860px;height:540px',
	origin: { x: 50, y: 54 },
	frames: [
		{ x: 96, y: 52, w: 440, h: 384 },
		{ x: 584, y: 52, w: 256, h: 384 }
	],
	onboarding: { x: 124, y: 84, w: 156, h: 124 },
	export: { x: 348, y: 168, w: 156, h: 124 },
	beta: { x: 634, y: 164, w: 156, h: 124 },
	extra: [
		{ x: 348, y: 316, w: 156, h: 100 },
		{ x: 634, y: 308, w: 156, h: 100 }
	],
	drag: { x: 16, y: 170 },
	arrow: 'M506 230 C 566 230, 572 226, 630 226',
	head: 'M622 219 630 226 622 233',
	from: [1, 0.5],
	to: [0, 0.5],
	toolbar: 'column'
};

/** The 358 × 620 design for a phone column. */
export const COMPACT: HeroLayout = {
	window: 'top:8px;left:8px;width:342px;height:540px',
	origin: { x: 8, y: 46 },
	frames: [
		{ x: 14, y: 32, w: 314, h: 236 },
		{ x: 14, y: 300, w: 314, h: 140 }
	],
	onboarding: { x: 30, y: 62, w: 128, h: 104 },
	export: { x: 182, y: 140, w: 128, h: 104 },
	beta: { x: 104, y: 320, w: 128, h: 100 },
	extra: [],
	drag: { x: 6, y: 86 },
	arrow: 'M246 246 C 246 310, 290 370, 238 370',
	head: 'M246 363 238 370 246 377',
	from: [0.5, 1],
	to: [1, 0.5],
	toolbar: 'row'
};

function at(layout: HeroLayout, box: Box, [fx, fy]: Anchor, shift: Spot = { x: 0, y: 0 }): Spot {
	return { x: layout.origin.x + box.x + box.w * fx + shift.x, y: layout.origin.y + box.y + box.h * fy + shift.y };
}

/** Where the three cursors rest in the poster frame, in stage pixels: everyone mid-action. */
export function pose(layout: HeroLayout): { you: Spot; lea: Spot; malik: Spot } {
	return {
		you: at(layout, layout.beta, layout.to),
		lea: at(layout, layout.onboarding, GRIP.lea, layout.drag),
		malik: at(layout, layout.export, GRIP.malik)
	};
}

/** Sticky fills, collaborator colours: muse's identity palette. */
export const COLORS = {
	ideas: USER_COLORS[0],
	export: USER_COLORS[4],
	extra: [USER_COLORS[2], USER_COLORS[0]],
	beta: USER_COLORS[3],
	lea: USER_COLORS[1],
	malik: USER_COLORS[5]
} as const;

/** The board toolbar of the demo, in the app's order and with the app's glyphs. */
export const TOOLS = [
	{ id: 'select', icon: toolIcons.select },
	{ id: 'sticky', icon: toolIcons.sticky },
	{ id: 'text', icon: toolIcons.text },
	{ id: 'rect', icon: toolIcons.rect },
	{ id: 'arrow', icon: toolIcons.arrow },
	{ id: 'pen', icon: toolIcons.pen }
] as const;

/** A sticky of the hero board someone can select. */
export type StickyId = 'onboarding' | 'export' | 'beta';

/** The three people on the hero board. */
export type Who = 'you' | 'lea' | 'malik';

/** Everything the hero loop changes on the board. */
export type BoardState = {
	tool: string;
	placed: boolean;
	beta: string;
	note: string;
	line: string;
	linked: boolean;
	moved: boolean;
	picks: Record<Who, StickyId | null>;
	cursors: Record<Who, CursorState>;
};

/** The elements the loop measures, read when each move starts. */
export type BoardNodes = {
	canvas?: HTMLElement;
	stickies: Partial<Record<StickyId, HTMLElement>>;
	tool: (id: string) => Element | undefined;
};

/** The strings the three people type, in the landing's locale. */
export function copy(): { beta: string; note: string; line: string } {
	return { beta: t('hero.beta'), note: t('hero.note'), line: t('hero.line') };
}

/** A fresh board: nobody has done anything and every cursor is hidden. */
export function blank(): BoardState {
	return {
		tool: 'select',
		placed: false,
		beta: '',
		note: '',
		line: '',
		linked: false,
		moved: false,
		picks: { you: null, lea: null, malik: null },
		cursors: { you: cursorAt(), lea: cursorAt(), malik: cursorAt() }
	};
}

/** Puts `board` back to `blank()` in place, keeping the cursors where they are. */
export function clear(board: BoardState): void {
	Object.assign(board, { ...blank(), cursors: board.cursors });
	for (const cursor of Object.values(board.cursors)) Object.assign(cursor, { visible: false, down: false });
}

/** The poster frame: Léa mid-drag, Malik mid-sentence, the visitor finishing an arrow. */
export function posed(board: BoardState): void {
	const words = copy();
	Object.assign(board, {
		tool: 'arrow',
		placed: true,
		beta: words.beta,
		line: words.line.slice(0, 8),
		linked: true,
		moved: true,
		picks: { you: null, lea: 'onboarding', malik: 'export' }
	});
}
