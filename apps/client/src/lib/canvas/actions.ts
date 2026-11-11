import { keyBetween } from '$lib/board/fractional';
import type { Shape, ShapeFields, ShapeType, Style } from '$lib/board/schema';
import { pathPoints } from './bounds';
import type { Editor, ToolStyle } from './editor.svelte';

/** A fractional index above every shape. */
export function topZ(editor: Editor): string {
	const order = editor.scene.order;
	return keyBetween(order.length ? order[order.length - 1].z : null, null);
}

/** A fractional index below every shape, where new frames go. */
export function bottomZ(editor: Editor): string {
	const order = editor.scene.order;
	return keyBetween(null, order.length ? order[0].z : null);
}

/** The style a new shape of `type` takes from the toolbar's current style. */
export function styleFor(type: ShapeType, current: ToolStyle): Style {
	if (type === 'sticky') return current.fill ? { fill: current.fill } : {};
	if (type === 'ink' || type === 'arrow' || type === 'line' || type === 'text') {
		return current.stroke ? { stroke: current.stroke } : {};
	}
	if (type === 'frame' || type === 'image' || type === 'card') return {};
	return {
		...(current.stroke ? { stroke: current.stroke } : {}),
		...(current.shapeFill ? { fill: current.shapeFill } : {})
	};
}

/** Creates a shape at the top of the stack with the current style; returns its id, or null when the board is full. */
export function createShape(editor: Editor, type: ShapeType, fields: ShapeFields, text = ''): string | null {
	const z = type === 'frame' ? bottomZ(editor) : topZ(editor);
	try {
		return editor.doc.create(type, { z, style: styleFor(type, editor.style), ...fields }, text);
	} catch {
		return null;
	}
}

/** Freezes the bound ends of arrows into their points, so deleting what they pointed at leaves them in place. */
function freezeArrows(editor: Editor, gone: Set<string>): void {
	for (const id of gone) {
		for (const arrowId of editor.scene.arrowsOf(id)) {
			const arrow = editor.scene.get(arrowId);
			if (!arrow || gone.has(arrowId)) continue;
			const points = pathPoints(arrow, editor.scene.get).flatMap((p) => [p.x - arrow.x, p.y - arrow.y]);
			editor.doc.update(arrowId, { points, rot: 0 });
		}
	}
}

/** Deletes the unlocked shapes of the selection, and the shapes inside deleted frames. */
export function deleteSelection(editor: Editor): void {
	if (!editor.editable) return;
	const gone = new Set(editor.selection.filter((id) => !editor.scene.get(id)?.locked));
	for (const shape of editor.scene.shapes.values()) if (shape.parent && gone.has(shape.parent)) gone.add(shape.id);
	if (!gone.size) return;
	editor.doc.transact(() => {
		freezeArrows(editor, gone);
		editor.doc.remove(gone);
	});
	editor.select([]);
}

function painted(shape: Shape, color: string | undefined | null, width: number | undefined): Style {
	const style: Style = { ...shape.style };
	if (width !== undefined) style.width = width;
	if (color === null) return style;
	const key =
		shape.type === 'sticky' || ['rect', 'ellipse', 'diamond', 'frame'].includes(shape.type) ? 'fill' : 'stroke';
	if (color === undefined) delete style[key];
	else style[key] = color;
	return style;
}

/**
 * Paints the unlocked selection: the fill of stickies, boxes and frames, the stroke of text, lines and ink. `undefined`
 * restores the default; `null` keeps colours and only sets `width`. New shapes take the same choice.
 */
export function paint(editor: Editor, color: string | undefined | null, width?: number): void {
	if (color !== null) editor.style = { ...editor.style, fill: color, shapeFill: color, stroke: color };
	if (width !== undefined) editor.style = { ...editor.style, width };
	if (!editor.editable) return;
	const changes = new Map<string, ShapeFields>();
	for (const id of editor.selection) {
		const shape = editor.scene.get(id);
		if (shape && !shape.locked) changes.set(id, { style: painted(shape, color, width) });
	}
	editor.doc.updateMany(changes);
}
