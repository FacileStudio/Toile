import { matchShortcut } from '$lib/shortcuts';
import type { ToolName } from './tools/tool';

/** A command the board runs by name. */
export type Run =
	| 'undo'
	| 'redo'
	| 'delete'
	| 'duplicate'
	| 'selectAll'
	| 'lock'
	| 'front'
	| 'back'
	| 'forward'
	| 'backward'
	| 'zoomIn'
	| 'zoomOut'
	| 'zoomReset'
	| 'zoomFit'
	| 'zoomSelection'
	| 'escape'
	| 'edit'
	| 'help'
	| 'search'
	| 'image'
	| 'minimap';

/** Every command a key or a button can run on the board. */
export type Command = { tool: ToolName } | { run: Run } | { nudge: [number, number] };

const TOOLS: ToolName[] = [
	'select',
	'hand',
	'sticky',
	'text',
	'rect',
	'ellipse',
	'diamond',
	'line',
	'arrow',
	'pen',
	'eraser',
	'frame',
	'comment'
];

const RUNS: Record<string, Run> = {
	'tool.image': 'image',
	'edit.undo': 'undo',
	'edit.redo': 'redo',
	'edit.duplicate': 'duplicate',
	'edit.delete': 'delete',
	'edit.selectAll': 'selectAll',
	'edit.deselect': 'escape',
	'edit.text': 'edit',
	'arrange.forward': 'forward',
	'arrange.backward': 'backward',
	'arrange.front': 'front',
	'arrange.back': 'back',
	'arrange.lock': 'lock',
	'view.zoomIn': 'zoomIn',
	'view.zoomOut': 'zoomOut',
	'view.zoomReset': 'zoomReset',
	'view.zoomFit': 'zoomFit',
	'view.zoomSelection': 'zoomSelection',
	'view.minimap': 'minimap',
	'view.search': 'search',
	'view.help': 'help'
};

const ARROWS: Record<string, [number, number]> = {
	ArrowLeft: [-1, 0],
	ArrowRight: [1, 0],
	ArrowUp: [0, -1],
	ArrowDown: [0, 1]
};

/** The command of a shared shortcut id (`$lib/shortcuts`), or null when the canvas does not bind it by key. */
export function commandOf(id: string): Command | null {
	const tool = id.startsWith('tool.') ? (id.slice(5) as ToolName) : null;
	if (tool && TOOLS.includes(tool)) return { tool };
	return RUNS[id] ? { run: RUNS[id] } : null;
}

/** The command a key event means on the board, or null. Arrows nudge 1 unit, 10 with Shift. */
export function commandFor(event: KeyboardEvent): Command | null {
	const arrow = ARROWS[event.key];
	const mod = event.metaKey || event.ctrlKey;
	if (arrow && !mod && !event.altKey)
		return { nudge: [arrow[0] * (event.shiftKey ? 10 : 1), arrow[1] * (event.shiftKey ? 10 : 1)] };
	const shortcut = matchShortcut(event);
	return shortcut ? commandOf(shortcut.id) : null;
}
