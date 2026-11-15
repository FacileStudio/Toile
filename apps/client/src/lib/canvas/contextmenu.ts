import { t } from '$lib/i18n/index.svelte';
import { icons, toast } from '@facile/muse';
import type { ContextMenuItem } from '$lib/components/ContextMenu.svelte';
import { TEXT_TYPES, type Shape } from '$lib/board/schema';
import { keyCaps } from '$lib/shortcuts';
import type { Editor } from './editor.svelte';
import type { CommandUi } from './commands';
import { runCommand } from './commands';
import { pasteInto } from './paste';
import { duplicate, serialize } from './clipboard';
import { createShape, deleteSelection } from './actions';
import { openCard, openShape } from './tools/select';
import { youtubeId } from './youtube';
import { restack } from './order';
import { zoomFit, zoomReset, zoomSelection } from './view';
import { toolIcons } from './icons';
import type { Point } from './camera';

function cutSelection(editor: Editor): void {
	const text = serialize(editor);
	if (!text) return;
	void navigator.clipboard.writeText(text);
	deleteSelection(editor);
}

function copySelection(editor: Editor): void {
	const text = serialize(editor);
	if (!text) return;
	void navigator.clipboard.writeText(text);
	toast.success(t('menu.copied'));
}

export function elementMenuItems(editor: Editor, ui: CommandUi, shape: Shape): ContextMenuItem[] {
	const shapes = editor.selection.map((id) => editor.scene.get(id)).filter((s): s is Shape => !!s);
	const single = shapes.length === 1 ? shapes[0] : null;
	const locked = shapes.every((s) => s.locked);
	const items: ContextMenuItem[] = [];

	if (single?.type === 'card' && single.url) {
		if (single.kind === 'youtube' && youtubeId(single.url))
			items.push({ label: t('menu.play'), icon: toolIcons.play, onselect: () => openCard(editor, single) });
		items.push(
			{
				label: t('style.openLink'),
				icon: icons.external,
				onselect: () => single.url && window.open(single.url, '_blank', 'noopener')
			},
			{
				label: t('style.copyLink'),
				icon: icons.copy,
				onselect: () => {
					void navigator.clipboard.writeText(single.url!);
					toast.success(t('style.linkCopied'));
				}
			},
			{ label: '', separator: true, onselect: () => {} }
		);
	}

	if (single && (TEXT_TYPES.has(single.type) || single.type === 'frame') && editor.editable && !single.locked) {
		items.push(
			{
				label: t('menu.editText'),
				icon: icons.edit,
				shortcut: t('menu.enter'),
				onselect: () => openShape(editor, single)
			},
			{ label: '', separator: true, onselect: () => {} }
		);
	}

	if (editor.editable) {
		items.push({
			label: t('menu.cut'),
			icon: toolIcons.cut,
			shortcut: keyCaps('Mod+X').join(''),
			onselect: () => cutSelection(editor)
		});
	}

	items.push({
		label: t('menu.copy'),
		icon: icons.copy,
		shortcut: keyCaps('Mod+C').join(''),
		onselect: () => copySelection(editor)
	});

	if (editor.editable) {
		items.push(
			{
				label: t('style.duplicate'),
				icon: icons.copy,
				shortcut: keyCaps('Mod+D').join(''),
				onselect: () => duplicate(editor)
			},
			{
				label: t('style.delete'),
				icon: icons.remove,
				tone: 'danger',
				shortcut: keyCaps('Delete').join(''),
				onselect: () => deleteSelection(editor)
			},
			{ label: '', separator: true, onselect: () => {} },
			{
				label: t('style.front'),
				icon: toolIcons.front,
				shortcut: keyCaps('Mod+]').join(''),
				onselect: () => restack(editor, 'front')
			},
			{
				label: t('style.back'),
				icon: toolIcons.back,
				shortcut: keyCaps('Mod+[').join(''),
				onselect: () => restack(editor, 'back')
			},
			{ label: '', separator: true, onselect: () => {} },
			{
				label: locked ? t('style.unlock') : t('style.lock'),
				icon: locked ? toolIcons.unlock : toolIcons.lock,
				shortcut: keyCaps('Mod+Shift+L').join(''),
				onselect: () => runCommand(editor, { run: 'lock' }, ui)
			}
		);
	}

	items.push({
		label: t('menu.zoomSelection'),
		icon: icons.search,
		shortcut: keyCaps('Shift+2').join(''),
		onselect: () => zoomSelection(editor)
	});

	return items;
}

async function pasteText(editor: Editor, world: Point): Promise<void> {
	const text = await navigator.clipboard.readText().catch(() => '');
	if (!text) return;
	const dt = {
		getData: (t: string) => (t === 'text/plain' ? text : ''),
		files: []
	} as unknown as DataTransfer;
	await pasteInto(editor, dt, world);
}

function insertShape(editor: Editor, type: 'sticky' | 'text' | 'rect' | 'frame', at: Point): void {
	const bounds =
		type === 'sticky'
			? { x: at.x - 100, y: at.y - 100, w: 200, h: 200 }
			: type === 'text'
				? { x: at.x - 100, y: at.y - 16, w: 200, h: 32 }
				: type === 'rect'
					? { x: at.x - 100, y: at.y - 75, w: 200, h: 150 }
					: { x: at.x - 200, y: at.y - 150, w: 400, h: 300 };
	const id = createShape(editor, type, bounds, type === 'frame' ? 'Cadre' : undefined);
	if (!id) return;
	editor.select([id]);
	if (type === 'sticky' || type === 'text') editor.edit(id);
}

export function canvasMenuItems(editor: Editor, ui: CommandUi, world: Point): ContextMenuItem[] {
	const items: ContextMenuItem[] = [];

	if (editor.editable) {
		items.push(
			{
				label: t('menu.paste'),
				icon: toolIcons.paste,
				shortcut: keyCaps('Mod+V').join(''),
				onselect: () => pasteText(editor, world)
			},
			{ label: '', separator: true, onselect: () => {} },
			{
				label: t('menu.addSticky'),
				icon: toolIcons.sticky,
				shortcut: 'S',
				onselect: () => insertShape(editor, 'sticky', world)
			},
			{
				label: t('menu.addText'),
				icon: toolIcons.text,
				shortcut: 'T',
				onselect: () => insertShape(editor, 'text', world)
			},
			{
				label: t('menu.addRect'),
				icon: toolIcons.rect,
				shortcut: 'R',
				onselect: () => insertShape(editor, 'rect', world)
			},
			{
				label: t('menu.addFrame'),
				icon: toolIcons.frame,
				shortcut: 'F',
				onselect: () => insertShape(editor, 'frame', world)
			},
			{ label: '', separator: true, onselect: () => {} }
		);
	}

	items.push(
		{
			label: t('menu.selectAll'),
			icon: toolIcons.select,
			shortcut: keyCaps('Mod+A').join(''),
			onselect: () => runCommand(editor, { run: 'selectAll' }, ui)
		},
		{
			label: t('menu.zoomFit'),
			icon: toolIcons.fit,
			shortcut: keyCaps('Shift+1').join(''),
			onselect: () => zoomFit(editor)
		},
		{
			label: t('menu.zoomReset'),
			icon: icons.search,
			shortcut: keyCaps('Shift+0').join(''),
			onselect: () => zoomReset(editor)
		}
	);

	return items;
}
