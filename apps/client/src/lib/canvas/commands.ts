import { deleteSelection } from './actions';
import { duplicate } from './clipboard';
import type { Editor } from './editor.svelte';
import { restack } from './order';
import type { Command, Run } from './shortcuts';
import { openShape } from './tools/select';
import { VIEW_TOOLS, type ToolName } from './tools/tool';
import { nudge, zoomBy, zoomFit, zoomReset, zoomSelection } from './view';

/** The page-level panels a command can open. */
export type CommandUi = { help: () => void; search: () => void; image: () => void; minimap: () => void };

/** Whether the role in `editor` may pick tool `name`. */
export function toolAllowed(editor: Editor, name: ToolName): boolean {
	if (editor.editable) return true;
	return VIEW_TOOLS.has(name) || (name === 'comment' && editor.commentable);
}

function toggleLock(editor: Editor): void {
	if (!editor.editable || !editor.selection.length) return;
	const lock = !editor.selection.every((id) => editor.scene.get(id)?.locked);
	editor.doc.updateMany(new Map(editor.selection.map((id) => [id, { locked: lock ? true : undefined }])));
}

const EDITS = new Set([
	'undo',
	'redo',
	'delete',
	'duplicate',
	'lock',
	'front',
	'back',
	'forward',
	'backward',
	'edit',
	'image'
]);

function escape(editor: Editor): void {
	if (editor.tool !== 'select') editor.tool = 'select';
	else editor.select([]);
}

function edit(editor: Editor): void {
	const shape = editor.selection.length === 1 ? editor.scene.get(editor.selection[0]) : null;
	if (shape) openShape(editor, shape);
}

const RUN: Record<Run, (editor: Editor, ui: CommandUi) => void> = {
	undo: (e) => e.undo.undo(),
	redo: (e) => e.undo.redo(),
	delete: deleteSelection,
	duplicate,
	selectAll: (e) => e.select(e.scene.order.filter((s) => !s.locked).map((s) => s.id)),
	lock: toggleLock,
	front: (e) => restack(e, 'front'),
	back: (e) => restack(e, 'back'),
	forward: (e) => restack(e, 'forward'),
	backward: (e) => restack(e, 'backward'),
	zoomIn: (e) => zoomBy(e, 1.25),
	zoomOut: (e) => zoomBy(e, 0.8),
	zoomReset,
	zoomFit,
	zoomSelection,
	escape,
	edit,
	help: (_e, ui) => ui.help(),
	search: (_e, ui) => ui.search(),
	image: (_e, ui) => ui.image(),
	minimap: (_e, ui) => ui.minimap()
};

/** Runs a command, refusing edits for a role that cannot edit. Returns whether it ran. */
export function runCommand(editor: Editor, command: Command, ui: CommandUi): boolean {
	if ('tool' in command) {
		if (!toolAllowed(editor, command.tool)) return false;
		editor.tool = command.tool;
		return true;
	}
	if ('nudge' in command) {
		if (!editor.selection.length) return false;
		nudge(editor, ...command.nudge);
		return true;
	}
	if (EDITS.has(command.run) && !editor.editable) return false;
	RUN[command.run](editor, ui);
	return true;
}
