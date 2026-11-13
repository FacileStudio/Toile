import { USER_COLORS, icons, registerIcons } from '@facile/muse';

/** A rectangle on the faux board, in the window body's design pixels. */
export type Box = { x: number; y: number; w: number; h: number };

/** A spot a collaborator's cursor rests on. */
export type Spot = { x: number; y: number };

/** Where everything sits in one design of the hero board. */
export type HeroLayout = {
	window: string;
	frames: [Box, Box];
	stickies: [Box, Box, Box];
	extra: Box[];
	arrow: string;
	head: string;
	lea: [Spot, Spot];
	malik: [Spot, Spot, Spot];
	toolbar: 'column' | 'row';
};

const point = (x: number, y: number): Spot => ({ x, y });

/** The 960 × 640 design under the hero copy. */
export const WIDE: HeroLayout = {
	window: 'top:16px;left:50px;width:860px;height:540px',
	frames: [
		{ x: 96, y: 52, w: 440, h: 384 },
		{ x: 584, y: 52, w: 256, h: 384 }
	],
	stickies: [
		{ x: 124, y: 84, w: 156, h: 124 },
		{ x: 348, y: 168, w: 156, h: 124 },
		{ x: 634, y: 164, w: 156, h: 124 }
	],
	extra: [
		{ x: 156, y: 284, w: 156, h: 120 },
		{ x: 634, y: 308, w: 156, h: 100 }
	],
	arrow: 'M506 230 C 566 230, 572 226, 630 226',
	head: 'M622 219 630 226 622 233',
	lea: [point(330, 470), point(770, 200)],
	malik: [point(760, 110), point(500, 236), point(632, 230)],
	toolbar: 'column'
};

/** The 358 × 620 design for a phone column. */
export const COMPACT: HeroLayout = {
	window: 'top:8px;left:8px;width:342px;height:540px',
	frames: [
		{ x: 14, y: 32, w: 314, h: 236 },
		{ x: 14, y: 300, w: 314, h: 140 }
	],
	stickies: [
		{ x: 30, y: 62, w: 128, h: 104 },
		{ x: 182, y: 140, w: 128, h: 104 },
		{ x: 104, y: 320, w: 128, h: 100 }
	],
	extra: [],
	arrow: 'M246 246 C 246 310, 290 370, 238 370',
	head: 'M246 363 238 370 246 377',
	lea: [point(120, 220), point(230, 330)],
	malik: [point(260, 40), point(250, 244), point(236, 372)],
	toolbar: 'row'
};

/** Sticky fills, collaborator colours: muse's identity palette. */
export const COLORS = {
	ideas: USER_COLORS[0],
	export: USER_COLORS[4],
	extra: [USER_COLORS[2], USER_COLORS[0]],
	beta: USER_COLORS[3],
	lea: USER_COLORS[1],
	malik: USER_COLORS[5]
} as const;

const SELECT = 'toile:tool-select';
const STICKY = 'toile:tool-sticky';
const SHAPE = 'toile:tool-shape';
const line = 'fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"';

registerIcons({
	[SELECT]: { body: `<path ${line} d="M6 4l12 6.5-5.2 1.6L10.5 17.5Z"/>`, width: 24, height: 24 },
	[STICKY]: { body: `<path ${line} d="M5 6a1 1 0 0 1 1-1h12a1 1 0 0 1 1 1v8l-5 5H6a1 1 0 0 1-1-1Z M14 19v-4a1 1 0 0 1 1-1h4"/>`, width: 24, height: 24 },
	[SHAPE]: { body: `<g ${line}><rect x="4" y="4" width="9" height="9" rx="1.5"/><circle cx="15.5" cy="15.5" r="4.5"/></g>`, width: 24, height: 24 }
});

/** The board toolbar of the demo, in order; `sticky` is the one the cursor picks. */
export const TOOLS = [
	{ id: 'select', icon: SELECT },
	{ id: 'sticky', icon: STICKY },
	{ id: 'text', icon: icons.text },
	{ id: 'shape', icon: SHAPE },
	{ id: 'pen', icon: icons.edit }
] as const;
