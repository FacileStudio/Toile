import { describe, expect, it } from 'vitest';
import { SHORTCUTS, SHORTCUT_GROUPS, keyCaps, matchShortcut } from './shortcuts';

const press = (key: string, init: KeyboardEventInit = {}) => new KeyboardEvent('keydown', { key, ...init });

describe('shortcuts', () => {
	it('binds every combo to one command and every command to a known group', () => {
		const combos = SHORTCUTS.flatMap((s) => s.keys);
		expect(new Set(combos).size).toBe(combos.length);
		expect(new Set(SHORTCUTS.map((s) => s.id)).size).toBe(SHORTCUTS.length);
		const groups = SHORTCUT_GROUPS.map((g) => g.id);
		expect(SHORTCUTS.every((s) => groups.includes(s.group))).toBe(true);
	});

	it('matches modifiers by platform', () => {
		expect(matchShortcut(press('z', { metaKey: true }), true)?.id).toBe('edit.undo');
		expect(matchShortcut(press('z', { ctrlKey: true }), false)?.id).toBe('edit.undo');
		expect(matchShortcut(press('z', { ctrlKey: true }), true)).toBeUndefined();
		expect(matchShortcut(press('Z', { metaKey: true, shiftKey: true }), true)?.id).toBe('edit.redo');
	});

	it('ignores shift on symbols and reads digits by position', () => {
		expect(matchShortcut(press('?', { shiftKey: true }), false)?.id).toBe('view.help');
		expect(matchShortcut(press('+', { ctrlKey: true, shiftKey: true }), false)?.id).toBe('view.zoomIn');
		expect(matchShortcut(press('0', { code: 'Digit0', shiftKey: true }), false)?.id).toBe('view.zoomReset');
		expect(matchShortcut(press(')', { code: 'Digit0', shiftKey: true }), false)?.id).toBe('view.zoomReset');
		expect(matchShortcut(press('s', { shiftKey: true }), false)).toBeUndefined();
		expect(matchShortcut(press(' '), false)?.id).toBe('view.pan');
	});

	it('names the keys for each platform', () => {
		expect(keyCaps('Mod+Shift+Z', true)).toEqual(['⌘', '⇧', 'Z']);
		expect(keyCaps('Mod+Shift+Z', false)).toEqual(['Ctrl', 'Maj', 'Z']);
		expect(keyCaps('Mod++', false)).toEqual(['Ctrl', '+']);
		expect(keyCaps('?', false)).toEqual(['?']);
	});
});
