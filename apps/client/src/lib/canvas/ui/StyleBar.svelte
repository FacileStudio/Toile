<script lang="ts">
	import { Icon, Tooltip, USER_COLOR_LABELS, icons, type UserColor } from '@facile/muse';
	import type { Editor } from '../editor.svelte';
	import { paint } from '../actions';
	import { runCommand, type CommandUi } from '../commands';
	import { toolIcons } from '../icons';
	import { FILLS } from '../render/palette';
	import type { Command } from '../shortcuts';

	let { editor, ui }: { editor: Editor; ui: CommandUi } = $props();

	const WIDTHS = [2, 4, 8];

	const shapes = $derived.by(() => {
		void editor.version;
		return editor.selection.map((id) => editor.scene.get(id)).filter((s) => !!s);
	});
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
		'toile-press flex size-11 shrink-0 items-center justify-center rounded-fc-pill text-fc-fg hover:bg-fc-surface focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-fc-ring';
</script>

{#if visible}
	<div
		class="toile-rise pointer-events-auto flex max-w-[calc(100vw-1.5rem)] items-center gap-1 overflow-x-auto rounded-fc-pill bg-fc-component p-1.5 shadow-lg"
		role="toolbar"
		aria-label="Style"
		data-board-ui
	>
		<div class="flex items-center" role="radiogroup" aria-label="Couleur">
			<button
				type="button"
				role="radio"
				aria-checked={!current}
				aria-label="Couleur par défaut"
				class="{button} {!current ? 'bg-fc-surface' : ''}"
				onclick={() => paint(editor, undefined)}
			>
				<span class="size-5 rounded-fc-pill bg-fc-fg"></span>
			</button>
			{#each FILLS as color, i (color)}
				<button
					type="button"
					role="radio"
					aria-checked={current === color}
					aria-label={USER_COLOR_LABELS[color as UserColor]}
					class="toile-pop {button} {current === color ? 'bg-fc-surface' : ''}"
					style:--i={i + 1}
					onclick={() => paint(editor, color)}
				>
					<span class="size-5 rounded-fc-pill" style:background-color={color}></span>
				</button>
			{/each}
		</div>
		{#if inky}
			<div class="flex items-center" role="radiogroup" aria-label="Épaisseur">
				{#each WIDTHS as size (size)}
					<button
						type="button"
						role="radio"
						aria-checked={width === size}
						aria-label="Épaisseur {size}"
						class="{button} {width === size ? 'bg-fc-surface' : ''}"
						onclick={() => paint(editor, null, size)}
					>
						<span class="rounded-fc-pill bg-fc-fg" style:width="{size + 2}px" style:height="{size + 2}px"></span>
					</button>
				{/each}
			</div>
		{/if}
		{#if shapes.length}
			<Tooltip text="Premier plan (⌘ ])">
				<button type="button" class={button} aria-label="Premier plan" onclick={() => run({ run: 'front' })}>
					<Icon icon={toolIcons.front} size={20} />
				</button>
			</Tooltip>
			<Tooltip text="Arrière-plan (⌘ [)">
				<button type="button" class={button} aria-label="Arrière-plan" onclick={() => run({ run: 'back' })}>
					<Icon icon={toolIcons.back} size={20} />
				</button>
			</Tooltip>
			<Tooltip text={locked ? 'Déverrouiller' : 'Verrouiller'}>
				<button
					type="button"
					class={button}
					aria-label={locked ? 'Déverrouiller' : 'Verrouiller'}
					onclick={() => run({ run: 'lock' })}
				>
					<Icon icon={locked ? toolIcons.unlock : toolIcons.lock} size={20} />
				</button>
			</Tooltip>
			<Tooltip text="Dupliquer (⌘ D)">
				<button type="button" class={button} aria-label="Dupliquer" onclick={() => run({ run: 'duplicate' })}>
					<Icon icon={icons.copy} size={20} />
				</button>
			</Tooltip>
			<Tooltip text="Supprimer">
				<button type="button" class={button} aria-label="Supprimer" onclick={() => run({ run: 'delete' })}>
					<Icon icon={icons.remove} size={20} />
				</button>
			</Tooltip>
		{/if}
	</div>
{/if}
