<script lang="ts">
	import { Modal } from '@facile/muse';
	import { SHORTCUTS, SHORTCUT_GROUPS, keyCaps } from '$lib/shortcuts';

	let { open = $bindable(false) }: { open?: boolean } = $props();

	const EXTRA = [
		{ label: 'Déplacer la sélection', keys: ['←', '↑', '→', '↓'] },
		{ label: 'Déplacer sans aimantation', keys: ['Alt'] },
		{ label: 'Zoomer', keys: ['Ctrl', 'molette'] }
	];
	const groups = SHORTCUT_GROUPS.map((group) => ({
		...group,
		rows: SHORTCUTS.filter((s) => s.group === group.id).map((s) => ({ label: s.label, keys: keyCaps(s.keys[0]) }))
	}));
</script>

{#snippet row(label: string, keys: string[])}
	<div class="flex items-center justify-between gap-4 py-1">
		<dt class="text-fc-sm text-fc-fg">{label}</dt>
		<dd class="flex shrink-0 gap-1">
			{#each keys as key, i (i)}
				<kbd class="rounded-fc-sm bg-fc-surface px-2 py-0.5 font-fc-mono text-fc-xs text-fc-fg-muted">{key}</kbd>
			{/each}
		</dd>
	</div>
{/snippet}

<Modal bind:open title="Raccourcis clavier" showClose size="lg">
	<div class="grid grid-cols-1 gap-6 sm:grid-cols-2">
		{#each groups as group (group.id)}
			<section class="flex flex-col gap-1">
				<h3 class="text-fc-xs font-medium text-fc-fg-muted">{group.label}</h3>
				<dl>
					{#each group.rows as r (r.label)}{@render row(r.label, r.keys)}{/each}
					{#if group.id === 'view'}
						{#each EXTRA as r (r.label)}{@render row(r.label, r.keys)}{/each}
					{/if}
				</dl>
			</section>
		{/each}
	</div>
</Modal>
