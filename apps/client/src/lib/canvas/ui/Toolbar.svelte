<script lang="ts">
	import { Icon, Tooltip } from '@facile/muse';
	import type { Editor } from '../editor.svelte';
	import { toolAllowed } from '../commands';
	import { toolIcons } from '../icons';
	import type { ToolName } from '../tools/tool';
	import { SHORTCUTS, keyCaps } from '$lib/shortcuts';

	let { editor, onimage }: { editor: Editor; onimage: () => void } = $props();

	const TOOLS = SHORTCUTS.filter((s) => s.group === 'tools' && s.id !== 'tool.image').map((s) => ({
		name: s.id.slice(5) as ToolName,
		label: s.label,
		key: keyCaps(s.keys[0]).join(' ')
	}));

	const shown = $derived(TOOLS.filter((tool) => toolAllowed(editor, tool.name)));
</script>

<div
	class="toile-rise pointer-events-auto flex max-w-[calc(100vw-1.5rem)] items-center gap-1 overflow-x-auto rounded-fc-pill bg-fc-component p-1.5 shadow-lg"
	role="toolbar"
	aria-label="Outils"
	data-board-ui
>
	{#each shown as tool, i (tool.name)}
		<Tooltip text="{tool.label} ({tool.key})">
			<button
				type="button"
				class="toile-pop toile-press flex size-11 shrink-0 items-center justify-center rounded-fc-pill cursor-pointer focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-fc-ring {editor.tool ===
				tool.name
					? 'bg-fc-accent text-fc-accent-fg'
					: 'text-fc-fg hover:bg-fc-surface'}"
				style:--i={i}
				aria-label={tool.label}
				aria-pressed={editor.tool === tool.name}
				onclick={() => (editor.tool = tool.name)}
			>
				<Icon icon={toolIcons[tool.name]} size={20} />
			</button>
		</Tooltip>
	{/each}
	{#if editor.editable}
		<Tooltip text="Image (I)">
			<button
				type="button"
				class="toile-pop toile-press flex size-11 shrink-0 items-center justify-center rounded-fc-pill cursor-pointer text-fc-fg hover:bg-fc-surface focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-fc-ring"
				style:--i={shown.length}
				aria-label="Image"
				onclick={onimage}
			>
				<Icon icon={toolIcons.image} size={20} />
			</button>
		</Tooltip>
	{/if}
</div>
