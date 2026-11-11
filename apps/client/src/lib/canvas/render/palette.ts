import { USER_COLORS } from '@facile/muse';

/** The muse tokens the canvas paints with, read from the live CSS so light and dark both follow the theme. */
export type Palette = {
	page: string;
	surface: string;
	component: string;
	fg: string;
	muted: string;
	accent: string;
	info: string;
	danger: string;
	font: string;
};

const TOKENS: Record<keyof Omit<Palette, 'font'>, string> = {
	page: '--color-fc-page',
	surface: '--color-fc-surface',
	component: '--color-fc-component',
	fg: '--color-fc-fg',
	muted: '--color-fc-fg-muted',
	accent: '--color-fc-accent',
	info: '--color-fc-info',
	danger: '--color-fc-danger'
};

/** Reads the palette from the document's computed tokens. */
export function readPalette(root: Element = document.documentElement): Palette {
	const style = getComputedStyle(root);
	const read = (name: string, fallback: string) => style.getPropertyValue(name).trim() || fallback;
	const out = { font: read('--font-fc-body', 'system-ui, sans-serif') } as Palette;
	for (const [key, token] of Object.entries(TOKENS)) out[key as keyof typeof TOKENS] = read(token, 'gray');
	return out;
}

/** The fill colours a sticky or shape can take: the suite identity palette, a data contract shared with Sablier. */
export const FILLS: readonly string[] = USER_COLORS;

/** The default sticky fill. */
export const STICKY_FILL = USER_COLORS[3];

/** Ink that stays legible on `hex`, from its luminance: the identity pastels all take dark ink. */
export function inkOn(hex: string): string {
	const value = /^#([0-9a-f]{6})$/i.exec(hex)?.[1];
	if (!value) return 'black';
	const [r, g, b] = [0, 2, 4].map((i) => parseInt(value.slice(i, i + 2), 16) / 255);
	return 0.2126 * r + 0.7152 * g + 0.0722 * b > 0.5 ? 'black' : 'white';
}

/** Watches the theme (OS scheme and the `.dark`/`.light` class) and calls `onchange` when it flips. */
export function watchTheme(onchange: () => void): () => void {
	const media = matchMedia('(prefers-color-scheme: dark)');
	const observer = new MutationObserver(onchange);
	observer.observe(document.documentElement, { attributes: true, attributeFilter: ['class', 'data-theme'] });
	media.addEventListener('change', onchange);
	return () => {
		observer.disconnect();
		media.removeEventListener('change', onchange);
	};
}
