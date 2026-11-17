<script lang="ts">
	import { t } from '$lib/i18n/index.svelte';
	import type { Editor } from '../editor.svelte';
	import { toolAllowed } from '../commands';
	import { toolIcons } from '../icons';
	import type { ToolName } from '@facile/planche';
	import { SHORTCUTS, keyCaps } from '$lib/shortcuts';
	import ToolGroup, { type ToolItem } from './ToolGroup.svelte';
	import type { AppKey } from '$lib/i18n/index.svelte';

	let { editor, onimage }: { editor: Editor; onimage: () => void } = $props();

	const SLOTS: { label: AppKey; ids: string[] }[] = [
		{ label: 'toolbar.select', ids: ['select', 'hand'] },
		{ label: 'toolbar.sticky', ids: ['sticky'] },
		{ label: 'toolbar.text', ids: ['text'] },
		{ label: 'toolbar.shapes', ids: ['rect', 'ellipse', 'diamond'] },
		{ label: 'toolbar.connectors', ids: ['arrow', 'line'] },
		{ label: 'toolbar.draw', ids: ['pen', 'eraser'] },
		{ label: 'toolbar.insert', ids: ['frame', 'image'] },
		{ label: 'toolbar.comment', ids: ['comment'] }
	];

	function item(id: string): ToolItem | null {
		const shortcut = SHORTCUTS.find((s) => s.id === `tool.${id}`);
		if (!shortcut) return null;
		const base = { id, label: shortcut.label, key: keyCaps(shortcut.keys[0]).join(' '), icon: toolIcons[id as keyof typeof toolIcons] };
		if (id === 'image') return editor.editable ? { ...base, run: onimage } : null;
		return toolAllowed(editor, id as ToolName) ? { ...base, tool: id as ToolName } : null;
	}

	const slots = $derived(
		SLOTS.map((slot) => ({ ...slot, label: t(slot.label), items: slot.ids.map(item).filter((i): i is ToolItem => !!i) })).filter(
			(slot) => slot.items.length
		)
	);
</script>

<div
	class="toile-rise pointer-events-auto flex max-w-full items-center gap-1 overflow-x-auto rounded-fc-pill bg-fc-component p-1.5 shadow-lg"
	role="toolbar"
	aria-label={t('toolbar.label')}
	data-board-ui
>
	{#each slots as slot, i (slot.label)}
		<ToolGroup {editor} label={slot.label} items={slot.items} index={i} />
	{/each}
</div>
