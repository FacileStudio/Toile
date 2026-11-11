import type { Editor } from '../editor.svelte';
import { CommentTool } from './comment';
import { ConnectorTool } from './connector';
import { CreateTool } from './create';
import { EraserTool } from './eraser';
import { HandTool } from './hand';
import { PenTool } from './pen';
import { SelectTool } from './select';
import { BOX_TOOLS, type Tool, type ToolName } from './tool';

/** A fresh gesture handler for tool `name`. */
export function makeTool(editor: Editor, name: ToolName): Tool {
	if (BOX_TOOLS.has(name)) return new CreateTool(editor, name as 'sticky');
	if (name === 'hand') return new HandTool(editor);
	if (name === 'pen') return new PenTool(editor);
	if (name === 'eraser') return new EraserTool(editor);
	if (name === 'arrow' || name === 'line') return new ConnectorTool(editor, name);
	if (name === 'comment') return new CommentTool(editor);
	return new SelectTool(editor);
}
