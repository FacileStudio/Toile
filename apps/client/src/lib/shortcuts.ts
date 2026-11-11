/** Where a shortcut is listed on the cheatsheet and in settings. */
export type ShortcutGroup = 'tools' | 'edit' | 'arrange' | 'view';

/**
 * One board command and its key combos. A combo is `+`-joined: modifiers first (`Mod` is ⌘ on
 * Apple platforms and Ctrl elsewhere, then `Shift`, `Alt`), the key last, as `KeyboardEvent.key`
 * spells it (`Space` for the space bar). The first combo is the one shown.
 */
export type Shortcut = { id: string; group: ShortcutGroup; label: string; keys: readonly string[] };

/** The headings of the groups, in display order. */
export const SHORTCUT_GROUPS: readonly { id: ShortcutGroup; label: string }[] = [
	{ id: 'tools', label: 'Outils' },
	{ id: 'edit', label: 'Édition' },
	{ id: 'arrange', label: 'Disposition' },
	{ id: 'view', label: 'Vue' }
];

/** Every keyboard shortcut of the board: the canvas binds these and settings lists them. */
export const SHORTCUTS: readonly Shortcut[] = [
	{ id: 'tool.select', group: 'tools', label: 'Sélection', keys: ['V'] },
	{ id: 'tool.hand', group: 'tools', label: 'Main', keys: ['H'] },
	{ id: 'tool.sticky', group: 'tools', label: 'Post-it', keys: ['S'] },
	{ id: 'tool.text', group: 'tools', label: 'Texte', keys: ['T'] },
	{ id: 'tool.rect', group: 'tools', label: 'Rectangle', keys: ['R'] },
	{ id: 'tool.ellipse', group: 'tools', label: 'Ellipse', keys: ['O'] },
	{ id: 'tool.diamond', group: 'tools', label: 'Losange', keys: ['D'] },
	{ id: 'tool.line', group: 'tools', label: 'Ligne', keys: ['L'] },
	{ id: 'tool.arrow', group: 'tools', label: 'Flèche', keys: ['A'] },
	{ id: 'tool.pen', group: 'tools', label: 'Crayon', keys: ['P'] },
	{ id: 'tool.eraser', group: 'tools', label: 'Gomme', keys: ['E'] },
	{ id: 'tool.image', group: 'tools', label: 'Image', keys: ['I'] },
	{ id: 'tool.frame', group: 'tools', label: 'Cadre', keys: ['F'] },
	{ id: 'tool.comment', group: 'tools', label: 'Commentaire', keys: ['C'] },
	{ id: 'edit.undo', group: 'edit', label: 'Annuler', keys: ['Mod+Z'] },
	{ id: 'edit.redo', group: 'edit', label: 'Rétablir', keys: ['Mod+Shift+Z', 'Mod+Y'] },
	{ id: 'edit.copy', group: 'edit', label: 'Copier', keys: ['Mod+C'] },
	{ id: 'edit.cut', group: 'edit', label: 'Couper', keys: ['Mod+X'] },
	{ id: 'edit.paste', group: 'edit', label: 'Coller', keys: ['Mod+V'] },
	{ id: 'edit.duplicate', group: 'edit', label: 'Dupliquer', keys: ['Mod+D'] },
	{ id: 'edit.delete', group: 'edit', label: 'Supprimer', keys: ['Delete', 'Backspace'] },
	{ id: 'edit.selectAll', group: 'edit', label: 'Tout sélectionner', keys: ['Mod+A'] },
	{ id: 'edit.text', group: 'edit', label: 'Modifier le texte', keys: ['Enter'] },
	{ id: 'edit.deselect', group: 'edit', label: 'Désélectionner', keys: ['Escape'] },
	{ id: 'arrange.forward', group: 'arrange', label: 'Avancer', keys: [']'] },
	{ id: 'arrange.backward', group: 'arrange', label: 'Reculer', keys: ['['] },
	{ id: 'arrange.front', group: 'arrange', label: 'Mettre au premier plan', keys: ['Mod+]'] },
	{ id: 'arrange.back', group: 'arrange', label: 'Mettre à l’arrière-plan', keys: ['Mod+['] },
	{ id: 'arrange.lock', group: 'arrange', label: 'Verrouiller ou déverrouiller', keys: ['Mod+Shift+L'] },
	{ id: 'view.pan', group: 'view', label: 'Déplacer la vue (maintenir)', keys: ['Space'] },
	{ id: 'view.zoomIn', group: 'view', label: 'Zoom avant', keys: ['Mod+=', 'Mod++'] },
	{ id: 'view.zoomOut', group: 'view', label: 'Zoom arrière', keys: ['Mod+-'] },
	{ id: 'view.zoomReset', group: 'view', label: 'Zoom à 100 %', keys: ['Shift+0'] },
	{ id: 'view.zoomFit', group: 'view', label: 'Tout afficher', keys: ['Shift+1'] },
	{ id: 'view.zoomSelection', group: 'view', label: 'Zoomer sur la sélection', keys: ['Shift+2'] },
	{ id: 'view.minimap', group: 'view', label: 'Mini-carte', keys: ['M'] },
	{ id: 'view.search', group: 'view', label: 'Rechercher dans le tableau', keys: ['Mod+K'] },
	{ id: 'view.help', group: 'view', label: 'Raccourcis clavier', keys: ['?'] }
];

type Combo = { mods: string[]; key: string };

const MAC_NAMES: Record<string, string> = { Mod: '⌘', Shift: '⇧', Alt: '⌥' };
const NAMES: Record<string, string> = {
	Mod: 'Ctrl',
	Shift: 'Maj',
	Alt: 'Alt',
	Delete: 'Suppr',
	Backspace: '⌫',
	Escape: 'Échap',
	Space: 'Espace'
};

/** Splits a combo into its modifiers and key; the key may itself be `+`. */
export function parseCombo(combo: string): Combo {
	const at = combo.lastIndexOf('+', combo.length - 2);
	return { mods: at < 0 ? [] : combo.slice(0, at).split('+'), key: combo.slice(at + 1) };
}

/** True on macOS and iOS, where `Mod` is ⌘. */
export function isApple(platform = typeof navigator === 'undefined' ? '' : navigator.platform): boolean {
	return /mac|iphone|ipad|ipod/i.test(platform);
}

/** The key caps of a combo, in display order: `['⌘', '⇧', 'Z']` on a Mac, `['Ctrl', 'Maj', 'Z']` elsewhere. */
export function keyCaps(combo: string, apple = isApple()): string[] {
	const { mods, key } = parseCombo(combo);
	return [...mods, key].map((part) => (apple && MAC_NAMES[part]) || NAMES[part] || part);
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
