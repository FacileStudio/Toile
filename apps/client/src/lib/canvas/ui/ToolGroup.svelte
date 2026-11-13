<script lang="ts" module>
	import type { ToolName } from '../tools/tool';

	/** One entry of a toolbar slot: a tool to switch to, or an action such as picking an image. */
	export type ToolItem = { id: string; label: string; key: string; icon: string; tool?: ToolName; run?: () => void };
</script>

<script lang="ts">
	import { tick } from 'svelte';
	import { Icon, Tooltip } from '@facile/muse';
	import type { Editor } from '../editor.svelte';

	let { editor, label, items, index }: { editor: Editor; label: string; items: ToolItem[]; index: number } = $props();

	const uid = $props.id();
	let open = $state(false);
	let picked = $state<string | null>(null);
	let button = $state<HTMLButtonElement>();
	let menu = $state<HTMLDivElement>();
	let spot = $state({ x: 0, y: 0 });

	const grouped = $derived(items.length > 1);
	const activeItem = $derived(items.find((item) => item.tool && item.tool === editor.tool));
	const current = $derived(activeItem ?? items.find((item) => item.id === picked) ?? items[0]);
	const active = $derived(!!activeItem);

	function choose(item: ToolItem) {
		picked = item.tool ? item.id : picked;
		if (item.tool) editor.tool = item.tool;
		item.run?.();
	}

	function press() {
		if (grouped && (active || !current.tool)) return toggle(!open);
		choose(current);
	}

	async function toggle(next: boolean, refocus = false) {
		open = next;
		if (!next) {
			if (refocus) button?.focus();
			return;
		}
		const box = button?.getBoundingClientRect();
		if (box) spot = { x: box.left + box.width / 2, y: window.innerHeight - box.top + 8 };
		await tick();
		menu?.showPopover?.();
		menu?.querySelector<HTMLElement>('[aria-checked="true"], [role="menuitemradio"]')?.focus();
	}

	function pick(item: ToolItem) {
		choose(item);
		void toggle(false, true);
	}

	function keydown(event: KeyboardEvent) {
		const entries = [...(menu?.querySelectorAll<HTMLElement>('[role="menuitemradio"]') ?? [])];
		const at = entries.indexOf(document.activeElement as HTMLElement);
		const step = { ArrowDown: 1, ArrowRight: 1, ArrowUp: -1, ArrowLeft: -1 }[event.key];
		if (step) entries[(at + step + entries.length) % entries.length]?.focus();
		if (event.key === 'Home') entries[0]?.focus();
		if (event.key === 'End') entries[entries.length - 1]?.focus();
		if (event.key === 'Escape') void toggle(false, true);
		if (step || ['Home', 'End', 'Escape'].includes(event.key)) {
			event.preventDefault();
			event.stopPropagation();
		}
	}

	$effect(() => {
		if (!open) return;
		const outside = (event: PointerEvent) => {
			const target = event.target as Node;
			if (!menu?.contains(target) && !button?.contains(target)) void toggle(false);
		};
		window.addEventListener('pointerdown', outside, true);
		return () => window.removeEventListener('pointerdown', outside, true);
	});

	const hint = $derived(grouped ? `${current.label} (${current.key}), ${label.toLowerCase()} : cliquer à nouveau pour choisir` : `${current.label} (${current.key})`);
</script>

<Tooltip text={hint} disabled={open}>
	<button
		bind:this={button}
		type="button"
		class="toile-pop toile-press relative flex size-11 shrink-0 cursor-pointer items-center justify-center rounded-fc-pill focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-fc-ring {active
			? 'bg-fc-accent text-fc-accent-fg'
			: 'text-fc-fg hover:bg-fc-surface'}"
		style:--i={index}
		aria-label={grouped ? `${label} : ${current.label}` : current.label}
		aria-pressed={current.tool ? active : undefined}
		aria-haspopup={grouped ? 'menu' : undefined}
		aria-expanded={grouped ? open : undefined}
		aria-controls={grouped && open ? `${uid}-menu` : undefined}
		onclick={press}
	>
		<Icon icon={current.icon} size={20} />
		{#if grouped}
			<span class="corner absolute right-1.5 bottom-1.5 size-1.5 bg-current opacity-60" aria-hidden="true"></span>
		{/if}
	</button>
</Tooltip>

{#if open}
	<div
		bind:this={menu}
		id="{uid}-menu"
		popover="manual"
		role="menu"
		aria-label={label}
		tabindex="-1"
		class="toile-rise m-0 flex flex-col gap-0.5 rounded-fc-lg bg-fc-component p-1.5 text-fc-fg shadow-lg"
		style:position="fixed"
		style:inset="auto"
		style:left="{spot.x}px"
		style:bottom="{spot.y}px"
		style:translate="-50% 0"
		onkeydown={keydown}
		data-board-ui
	>
		{#each items as item (item.id)}
			<button
				type="button"
				role="menuitemradio"
				aria-checked={item.id === current.id}
				tabindex="-1"
				class="flex min-h-11 cursor-pointer items-center gap-3 rounded-fc-md px-3 text-left text-fc-sm focus-visible:outline-2 focus-visible:outline-fc-ring {item.id ===
				current.id
					? 'bg-fc-surface'
					: 'hover:bg-fc-surface'}"
				onclick={() => pick(item)}
			>
				<Icon icon={item.icon} size={20} />
				<span class="flex-1">{item.label}</span>
				<kbd class="font-sans text-fc-xs text-fc-fg-muted">{item.key}</kbd>
			</button>
		{/each}
	</div>
{/if}

<style>
	.corner {
		clip-path: polygon(100% 0, 100% 100%, 0 100%);
	}
</style>
