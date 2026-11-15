<script lang="ts">
	import { t } from '$lib/i18n/index.svelte';
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
	aria-label={t('zoom.label')}
	data-board-ui
>
	{#if editor.editable}
		<Tooltip text={t('zoom.undoTip')} class="contents">
			<button type="button" class={button} aria-label={t('zoom.undo')} onclick={() => editor.undo.undo()}>
				<Icon icon={toolIcons.undo} size={20} />
			</button>
		</Tooltip>
		<Tooltip text={t('zoom.redoTip')} class="contents">
			<button type="button" class={button} aria-label={t('zoom.redo')} onclick={() => editor.undo.redo()}>
				<Icon icon={toolIcons.redo} size={20} />
			</button>
		</Tooltip>
	{/if}
	<Tooltip text={t('zoom.outTip')} class="contents">
		<button type="button" class={button} aria-label={t('zoom.out')} onclick={() => zoomBy(editor, 0.8)}>
			<Icon icon={icons.minus} size={20} />
		</button>
	</Tooltip>
	<Tooltip text={t('zoom.resetTip')} class="contents">
		<button
			type="button"
			class="h-11 min-w-14 whitespace-nowrap rounded-fc-pill px-2 text-fc-sm font-medium text-fc-fg tabular-nums hover:bg-fc-surface focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-fc-ring cursor-pointer"
			aria-label={t('zoom.current', { percent })}
			onclick={() => zoomReset(editor)}
		>
			{percent} %
		</button>
	</Tooltip>
	<Tooltip text={t('zoom.inTip')} class="contents">
		<button type="button" class={button} aria-label={t('zoom.in')} onclick={() => zoomBy(editor, 1.25)}>
			<Icon icon={icons.plus} size={20} />
		</button>
	</Tooltip>
	<Tooltip text={t('zoom.fitTip')} class="contents">
		<button type="button" class={button} aria-label={t('zoom.fit')} onclick={() => zoomFit(editor)}>
			<Icon icon={toolIcons.fit} size={20} />
		</button>
	</Tooltip>
</div>
