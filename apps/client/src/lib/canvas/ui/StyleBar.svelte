<script lang="ts">
	import { t, type AppKey } from '$lib/i18n/index.svelte';
	import { Icon, Tooltip, USER_COLOR_LABELS, icons, toast, type UserColor } from '@facile/muse';
	import type { Editor } from '../editor.svelte';
	import { paint } from '../actions';
	import { runCommand, type CommandUi } from '../commands';
	import { toolIcons } from '../icons';
	import { FILLS } from '../render/palette';
	import type { Command } from '../shortcuts';

	let { editor, ui }: { editor: Editor; ui: CommandUi } = $props();

	const WIDTHS = [2, 4, 8];
	const WIDTH_LABELS = $derived<Record<number, string>>({ 2: t('style.thin'), 4: t('style.medium'), 8: t('style.thick') });
	const colorName = (color: string) => t(`color.${USER_COLOR_LABELS[color as UserColor].toLowerCase()}` as AppKey);

	const shapes = $derived.by(() => {
		void editor.version;
		return editor.selection.map((id) => editor.scene.get(id)).filter((s) => !!s);
	});
	const isCard = $derived(shapes.length === 1 && shapes[0].type === 'card');
	const drawing = $derived(['sticky', 'rect', 'ellipse', 'diamond', 'text', 'arrow', 'pen'].includes(editor.tool));
	const inky = $derived(
		editor.tool === 'pen' || shapes.some((s) => s.type === 'ink' || s.type === 'arrow' || s.type === 'line')
	);
	const locked = $derived(shapes.length > 0 && shapes.every((s) => s.locked));
	const visible = $derived(editor.editable && !editor.editing && (shapes.length > 0 || drawing));
	const current = $derived(
		shapes.length === 1 ? (shapes[0].style.fill ?? shapes[0].style.stroke) : editor.style.stroke
	);
	const width = $derived(shapes.find((s) => s.style.width)?.style.width ?? editor.style.width ?? 4);

	const run = (command: Command) => runCommand(editor, command, ui);
	const button =
		'toile-press flex size-11 shrink-0 items-center justify-center rounded-fc-pill text-fc-fg hover:bg-fc-surface focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-fc-ring cursor-pointer disabled:cursor-not-allowed';
</script>

{#if visible}
	<div
		class="toile-rise pointer-events-auto flex max-w-[calc(100vw-1.5rem)] items-center gap-1 overflow-x-auto rounded-fc-pill bg-fc-component p-1.5 shadow-lg"
		role="toolbar"
		aria-label={t('style.label')}
		data-board-ui
	>
		{#if isCard}
			{#if shapes[0].url}
				<Tooltip text={t('style.openLink')} describe={false} class="contents">
					<button
						type="button"
						class={button}
						aria-label={t('style.openLink')}
						onclick={() => shapes[0].url && window.open(shapes[0].url, '_blank', 'noopener')}
					>
						<Icon icon={icons.external} size={20} />
					</button>
				</Tooltip>
				<Tooltip text={t('style.copyLink')} describe={false} class="contents">
					<button
						type="button"
						class={button}
						aria-label={t('style.copyLink')}
						onclick={() => {
							if (shapes[0].url) {
								void navigator.clipboard.writeText(shapes[0].url);
								toast.success(t('style.linkCopied'));
							}
						}}
					>
						<Icon icon={icons.copy} size={20} />
					</button>
				</Tooltip>
			{/if}
		{:else}
			<div class="flex items-center" role="radiogroup" aria-label={t('style.color')}>
				<Tooltip text={t('style.defaultColor')} describe={false} class="contents">
					<button
						type="button"
						role="radio"
						aria-checked={!current}
						aria-label={t('style.defaultColor')}
						class="{button} {!current ? 'bg-fc-surface' : ''}"
						onclick={() => paint(editor, undefined)}
					>
						<span class="size-5 rounded-fc-pill bg-fc-fg"></span>
					</button>
				</Tooltip>
				{#each FILLS as color, i (color)}
					<Tooltip text={colorName(color)} describe={false} class="contents">
						<button
							type="button"
							role="radio"
							aria-checked={current === color}
							aria-label={colorName(color)}
							class="toile-pop {button} {current === color ? 'bg-fc-surface' : ''}"
							style:--i={i + 1}
							onclick={() => paint(editor, color)}
						>
							<span class="size-5 rounded-fc-pill" style:background-color={color}></span>
						</button>
					</Tooltip>
				{/each}
			</div>
		{/if}
		{#if inky}
			<div class="flex items-center" role="radiogroup" aria-label={t('style.width')}>
				{#each WIDTHS as size (size)}
					<Tooltip text={WIDTH_LABELS[size]} class="contents">
						<button
							type="button"
							role="radio"
							aria-checked={width === size}
							aria-label={t('style.widthNamed', { width: WIDTH_LABELS[size].toLowerCase() })}
							class="{button} {width === size ? 'bg-fc-surface' : ''}"
							onclick={() => paint(editor, null, size)}
						>
							<span class="rounded-fc-pill bg-fc-fg" style:width="{size + 2}px" style:height="{size + 2}px"></span>
						</button>
					</Tooltip>
				{/each}
			</div>
		{/if}
		{#if shapes.length}
			<Tooltip text={t('style.frontTip')} class="contents">
				<button type="button" class={button} aria-label={t('style.front')} onclick={() => run({ run: 'front' })}>
					<Icon icon={toolIcons.front} size={20} />
				</button>
			</Tooltip>
			<Tooltip text={t('style.backTip')} class="contents">
				<button type="button" class={button} aria-label={t('style.back')} onclick={() => run({ run: 'back' })}>
					<Icon icon={toolIcons.back} size={20} />
				</button>
			</Tooltip>
			<Tooltip text={locked ? t('style.unlock') : t('style.lock')} describe={false} class="contents">
				<button
					type="button"
					class={button}
					aria-label={locked ? t('style.unlock') : t('style.lock')}
					onclick={() => run({ run: 'lock' })}
				>
					<Icon icon={locked ? toolIcons.unlock : toolIcons.lock} size={20} />
				</button>
			</Tooltip>
			<Tooltip text={t('style.duplicateTip')} class="contents">
				<button type="button" class={button} aria-label={t('style.duplicate')} onclick={() => run({ run: 'duplicate' })}>
					<Icon icon={icons.copy} size={20} />
				</button>
			</Tooltip>
			<Tooltip text={t('style.deleteTip')} class="contents">
				<button type="button" class={button} aria-label={t('style.delete')} onclick={() => run({ run: 'delete' })}>
					<Icon icon={icons.remove} size={20} />
				</button>
			</Tooltip>
		{/if}
	</div>
{/if}
