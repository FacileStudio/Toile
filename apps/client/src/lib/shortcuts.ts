import { t, type AppKey } from '$lib/i18n/index.svelte';

/** Where a shortcut is listed on the cheatsheet and in settings. */
export type ShortcutGroup = 'tools' | 'edit' | 'arrange' | 'view';

/**
 * One board command and its key combos. A combo is `+`-joined: modifiers first (`Mod` is ⌘ on
 * Apple platforms and Ctrl elsewhere, then `Shift`, `Alt`), the key last, as `KeyboardEvent.key`
 * spells it (`Space` for the space bar). The first combo is the one shown.
 */
export type Shortcut = { id: string; group: ShortcutGroup; label: string; keys: readonly string[] };

/** Gives each entry a `label` read from the catalog at `<prefix>.<id>`, in the locale in force when it is read. */
function withLabels<T extends { id: string }>(prefix: string, entries: T[]): (T & { label: string })[] {
	return entries.map((entry) => ({
		...entry,
		get label() {
			return t(`${prefix}.${entry.id}` as AppKey);
		}
	}));
}

/** The headings of the groups, in display order. */
export const SHORTCUT_GROUPS: readonly { id: ShortcutGroup; label: string }[] = withLabels<{ id: ShortcutGroup }>('shortcuts.group', [
	{ id: 'tools' },
	{ id: 'edit' },
	{ id: 'arrange' },
	{ id: 'view' }
]);

/** Every keyboard shortcut of the board: the canvas binds these and settings lists them. */
export const SHORTCUTS: readonly Shortcut[] = withLabels<Omit<Shortcut, 'label'>>('shortcut', [
	{ id: 'tool.select', group: 'tools', keys: ['V'] },
	{ id: 'tool.hand', group: 'tools', keys: ['H'] },
	{ id: 'tool.sticky', group: 'tools', keys: ['S'] },
	{ id: 'tool.text', group: 'tools', keys: ['T'] },
	{ id: 'tool.rect', group: 'tools', keys: ['R'] },
	{ id: 'tool.ellipse', group: 'tools', keys: ['O'] },
	{ id: 'tool.diamond', group: 'tools', keys: ['D'] },
	{ id: 'tool.line', group: 'tools', keys: ['L'] },
	{ id: 'tool.arrow', group: 'tools', keys: ['A'] },
	{ id: 'tool.pen', group: 'tools', keys: ['P'] },
	{ id: 'tool.eraser', group: 'tools', keys: ['E'] },
	{ id: 'tool.image', group: 'tools', keys: ['I'] },
	{ id: 'tool.frame', group: 'tools', keys: ['F'] },
	{ id: 'tool.comment', group: 'tools', keys: ['C'] },
	{ id: 'edit.undo', group: 'edit', keys: ['Mod+Z'] },
	{ id: 'edit.redo', group: 'edit', keys: ['Mod+Shift+Z', 'Mod+Y'] },
	{ id: 'edit.copy', group: 'edit', keys: ['Mod+C'] },
	{ id: 'edit.cut', group: 'edit', keys: ['Mod+X'] },
	{ id: 'edit.paste', group: 'edit', keys: ['Mod+V'] },
	{ id: 'edit.duplicate', group: 'edit', keys: ['Mod+D'] },
	{ id: 'edit.delete', group: 'edit', keys: ['Delete', 'Backspace'] },
	{ id: 'edit.selectAll', group: 'edit', keys: ['Mod+A'] },
	{ id: 'edit.text', group: 'edit', keys: ['Enter'] },
	{ id: 'edit.deselect', group: 'edit', keys: ['Escape'] },
	{ id: 'arrange.forward', group: 'arrange', keys: [']'] },
	{ id: 'arrange.backward', group: 'arrange', keys: ['['] },
	{ id: 'arrange.front', group: 'arrange', keys: ['Mod+]'] },
	{ id: 'arrange.back', group: 'arrange', keys: ['Mod+['] },
	{ id: 'arrange.lock', group: 'arrange', keys: ['Mod+Shift+L'] },
	{ id: 'view.pan', group: 'view', keys: ['Space'] },
	{ id: 'view.zoomIn', group: 'view', keys: ['Mod+=', 'Mod++'] },
	{ id: 'view.zoomOut', group: 'view', keys: ['Mod+-'] },
	{ id: 'view.zoomReset', group: 'view', keys: ['Shift+0'] },
	{ id: 'view.zoomFit', group: 'view', keys: ['Shift+1'] },
	{ id: 'view.zoomSelection', group: 'view', keys: ['Shift+2'] },
	{ id: 'view.minimap', group: 'view', keys: ['M'] },
	{ id: 'view.search', group: 'view', keys: ['Mod+K'] },
	{ id: 'view.help', group: 'view', keys: ['?'] }
]);

type Combo = { mods: string[]; key: string };

const MAC_NAMES: Record<string, string> = { Mod: '⌘', Shift: '⇧', Alt: '⌥' };
const NAMES: Record<string, string> = { Mod: 'Ctrl', Alt: 'Alt', Backspace: '⌫' };
const SPOKEN = new Set(['Shift', 'Delete', 'Escape', 'Space']);

function keyName(part: string): string {
	return SPOKEN.has(part) ? t(`key.${part}` as AppKey) : (NAMES[part] ?? part);
}

/** Splits a combo into its modifiers and key; the key may itself be `+`. */
export function parseCombo(combo: string): Combo {
	const at = combo.lastIndexOf('+', combo.length - 2);
	return { mods: at < 0 ? [] : combo.slice(0, at).split('+'), key: combo.slice(at + 1) };
}

/** True on macOS and iOS, where `Mod` is ⌘. */
export function isApple(platform = typeof navigator === 'undefined' ? '' : navigator.platform): boolean {
	return /mac|iphone|ipad|ipod/i.test(platform);
}

/** The key caps of a combo, in display order: `['⌘', '⇧', 'Z']` on a Mac, `['Ctrl', 'Shift', 'Z']` elsewhere, in the current locale. */
export function keyCaps(combo: string, apple = isApple()): string[] {
	const { mods, key } = parseCombo(combo);
	return [...mods, key].map((part) => (apple && MAC_NAMES[part]) || keyName(part));
}

function eventKey(event: KeyboardEvent): string {
	if (/^Digit\d$/.test(event.code)) return event.code.slice(5);
	return event.key === ' ' ? 'space' : event.key.toLowerCase();
}

function hits(combo: string, event: KeyboardEvent, apple: boolean): boolean {
	const { mods, key } = parseCombo(combo);
	const shift = mods.includes('Shift');
	const mod = apple ? event.metaKey : event.ctrlKey;
	if (eventKey(event) !== key.toLowerCase() || mod !== mods.includes('Mod')) return false;
	if (event.altKey !== mods.includes('Alt')) return false;
	return /^[^a-z0-9]$/i.test(key) && !shift ? true : event.shiftKey === shift;
}

/** The shortcut a key press triggers, if any. Shift is ignored for symbol keys like `?` or `+`; digits match by position. */
export function matchShortcut(event: KeyboardEvent, apple = isApple()): Shortcut | undefined {
	return SHORTCUTS.find((shortcut) => shortcut.keys.some((combo) => hits(combo, event, apple)));
}
