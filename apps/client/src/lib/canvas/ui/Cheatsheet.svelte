<script lang="ts">
	import { Modal } from '@facile/muse';
	import { SHORTCUTS, SHORTCUT_GROUPS, keyCaps } from '$lib/shortcuts';
	import { t } from '$lib/i18n/index.svelte';

	let { open = $bindable(false) }: { open?: boolean } = $props();

	const extra = $derived([
		{ label: t('cheatsheet.move'), keys: ['←', '↑', '→', '↓'] },
		{ label: t('cheatsheet.noSnap'), keys: ['Alt'] },
		{ label: t('cheatsheet.zoom'), keys: ['Ctrl', t('cheatsheet.wheel')] }
	]);
	const groups = $derived(
		SHORTCUT_GROUPS.map((group) => ({
			id: group.id,
			label: group.label,
			rows: SHORTCUTS.filter((s) => s.group === group.id).map((s) => ({ label: s.label, keys: keyCaps(s.keys[0]) }))
		}))
	);
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

<Modal bind:open title={t('cheatsheet.title')} showClose size="lg">
	<div class="grid grid-cols-1 gap-6 sm:grid-cols-2">
		{#each groups as group (group.id)}
			<section class="flex flex-col gap-1">
				<h3 class="text-fc-xs font-medium text-fc-fg-muted">{group.label}</h3>
				<dl>
					{#each group.rows as r (r.label)}{@render row(r.label, r.keys)}{/each}
					{#if group.id === 'view'}
						{#each extra as r (r.label)}{@render row(r.label, r.keys)}{/each}
					{/if}
				</dl>
			</section>
		{/each}
	</div>
</Modal>
