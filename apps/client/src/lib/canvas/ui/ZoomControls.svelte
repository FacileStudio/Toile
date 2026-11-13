<script lang="ts">
	import { Icon, Tooltip, icons } from '@facile/muse';
	import type { Editor } from '../editor.svelte';
	import { toolIcons } from '../icons';
	import { zoomBy, zoomFit, zoomReset } from '../view';

	let { editor }: { editor: Editor } = $props();

	const button =
		'toile-press flex size-11 shrink-0 items-center justify-center rounded-fc-pill text-fc-fg hover:bg-fc-surface disabled:opacity-50 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-fc-ring cursor-pointer disabled:cursor-not-allowed';
	const percent = $derived(Math.round(editor.camera.z * 100));
</script>

<div
	class="toile-rise pointer-events-auto flex items-center gap-0.5 rounded-fc-pill bg-fc-component p-1.5 shadow-lg"
	role="group"
	aria-label="Affichage"
	data-board-ui
>
	{#if editor.editable}
		<Tooltip text="Annuler (⌘ Z)" class="contents">
			<button type="button" class={button} aria-label="Annuler" onclick={() => editor.undo.undo()}>
				<Icon icon={toolIcons.undo} size={20} />
			</button>
		</Tooltip>
		<Tooltip text="Rétablir (⌘ ⇧ Z)" class="contents">
			<button type="button" class={button} aria-label="Rétablir" onclick={() => editor.undo.redo()}>
				<Icon icon={toolIcons.redo} size={20} />
			</button>
		</Tooltip>
	{/if}
	<Tooltip text="Zoom arrière (⌘ −)" class="contents">
		<button type="button" class={button} aria-label="Zoom arrière" onclick={() => zoomBy(editor, 0.8)}>
			<Icon icon={icons.minus} size={20} />
		</button>
	</Tooltip>
	<Tooltip text="Revenir à 100 % (⇧ 0)" class="contents">
		<button
			type="button"
			class="h-11 min-w-14 whitespace-nowrap rounded-fc-pill px-2 text-fc-sm font-medium text-fc-fg tabular-nums hover:bg-fc-surface focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-fc-ring cursor-pointer"
			aria-label="Zoom {percent} %, revenir à 100 %"
			onclick={() => zoomReset(editor)}
		>
			{percent} %
		</button>
	</Tooltip>
	<Tooltip text="Zoom avant (⌘ +)" class="contents">
		<button type="button" class={button} aria-label="Zoom avant" onclick={() => zoomBy(editor, 1.25)}>
			<Icon icon={icons.plus} size={20} />
		</button>
	</Tooltip>
	<Tooltip text="Tout afficher (⇧ 1)" class="contents">
		<button type="button" class={button} aria-label="Tout afficher" onclick={() => zoomFit(editor)}>
			<Icon icon={toolIcons.fit} size={20} />
		</button>
	</Tooltip>
</div>
