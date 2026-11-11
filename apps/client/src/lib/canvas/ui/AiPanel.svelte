<script lang="ts">
	import { Button, Icon, IconButton, Skeleton, icons } from '@facile/muse';
	import { blocks } from '$lib/markdown';

	let {
		markdown,
		busy,
		onretry,
		onclose
	}: { markdown: string; busy: boolean; onretry: () => void; onclose: () => void } = $props();

	const content = $derived(blocks(markdown));
	const heading = { 2: 'text-fc-md', 3: 'text-fc-sm', 4: 'text-fc-sm' } as const;
</script>

<aside
	class="pointer-events-auto flex max-h-[70dvh] w-full flex-col gap-4 overflow-y-auto rounded-fc-lg bg-fc-component p-4 shadow-lg md:max-h-full md:w-80"
	aria-label="Résumé du tableau"
	aria-busy={busy}
	data-board-ui
	data-testid="ai-summary"
>
	<header class="flex items-center justify-between gap-2">
		<h2 class="text-fc-md font-semibold text-fc-fg">Résumé</h2>
		<IconButton variant="ghost" aria-label="Fermer le résumé" onclick={onclose}>
			<Icon icon={icons.close} size={18} />
		</IconButton>
	</header>
	{#if busy}
		<div class="flex flex-col gap-2" aria-live="polite">
			<span class="sr-only">L’IA résume le tableau…</span>
			<Skeleton class="h-4 w-2/3" />
			<Skeleton class="h-4 w-full" />
			<Skeleton class="h-4 w-5/6" />
		</div>
	{:else}
		<div class="flex flex-col gap-3 text-fc-sm text-fc-fg" aria-live="polite">
			{#each content as block, i (i)}
				{#if block.kind === 'heading'}
					<svelte:element this={`h${block.level + 1}`} class="{heading[block.level]} font-semibold">{block.text}</svelte:element>
				{:else if block.kind === 'list'}
					<ul class="flex list-disc flex-col gap-1 pl-5">
						{#each block.items as item, n (n)}<li>{item}</li>{/each}
					</ul>
				{:else}
					<p>{block.text}</p>
				{/if}
			{/each}
		</div>
		<Button variant="secondary" size="sm" onclick={onretry}>Résumer à nouveau</Button>
	{/if}
</aside>
