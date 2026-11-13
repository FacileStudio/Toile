<script lang="ts">
	import type { Editor } from '../editor.svelte';
	import { toolAllowed } from '../commands';
	import { toolIcons } from '../icons';
	import type { ToolName } from '../tools/tool';
	import { SHORTCUTS, keyCaps } from '$lib/shortcuts';
	import ToolGroup, { type ToolItem } from './ToolGroup.svelte';

	let { editor, onimage }: { editor: Editor; onimage: () => void } = $props();

	const SLOTS: { label: string; ids: string[] }[] = [
		{ label: 'Sélection', ids: ['select', 'hand'] },
		{ label: 'Post-it', ids: ['sticky'] },
		{ label: 'Texte', ids: ['text'] },
		{ label: 'Formes', ids: ['rect', 'ellipse', 'diamond'] },
		{ label: 'Connecteurs', ids: ['arrow', 'line'] },
		{ label: 'Dessin', ids: ['pen', 'eraser'] },
		{ label: 'Insérer', ids: ['frame', 'image'] },
		{ label: 'Commentaire', ids: ['comment'] }
	];

	function item(id: string): ToolItem | null {
		const shortcut = SHORTCUTS.find((s) => s.id === `tool.${id}`);
		if (!shortcut) return null;
		const base = { id, label: shortcut.label, key: keyCaps(shortcut.keys[0]).join(' '), icon: toolIcons[id as keyof typeof toolIcons] };
		if (id === 'image') return editor.editable ? { ...base, run: onimage } : null;
		return toolAllowed(editor, id as ToolName) ? { ...base, tool: id as ToolName } : null;
	}

	const slots = $derived(
		SLOTS.map((slot) => ({ ...slot, items: slot.ids.map(item).filter((i): i is ToolItem => !!i) })).filter(
			(slot) => slot.items.length
		)
	);
</script>

<div
	class="toile-rise pointer-events-auto flex max-w-full items-center gap-1 overflow-x-auto rounded-fc-pill bg-fc-component p-1.5 shadow-lg"
	role="toolbar"
	aria-label="Outils"
	data-board-ui
>
	{#each slots as slot, i (slot.label)}
		<ToolGroup {editor} label={slot.label} items={slot.items} index={i} />
	{/each}
</div>
