<script lang="ts">
	import { SettingsSection } from '@facile/muse';
	import { SHORTCUTS, SHORTCUT_GROUPS, isApple, keyCaps } from '$lib/shortcuts';

	const apple = isApple();
	const groups = SHORTCUT_GROUPS.map((group) => ({
		...group,
		items: SHORTCUTS.filter((shortcut) => shortcut.group === group.id)
	}));
</script>

<!--
@component
The board's keyboard shortcuts, read from `$lib/shortcuts`, one section per group. On a board,
`?` opens the same list.
-->

<div class="flex flex-col gap-8">
	{#each groups as group (group.id)}
		<SettingsSection title={group.label}>
			<dl class="grid gap-x-8 sm:grid-cols-2">
				{#each group.items as shortcut (shortcut.id)}
					<div class="flex min-h-11 items-center justify-between gap-4 py-1">
						<dt class="text-fc-sm text-fc-fg">{shortcut.label}</dt>
						<dd class="flex flex-wrap items-center justify-end gap-2">
							{#each shortcut.keys as combo, at (combo)}
								{#if at > 0}<span class="text-fc-xs text-fc-fg-muted">ou</span>{/if}
								<span class="flex items-center gap-1">
									{#each keyCaps(combo, apple) as cap, index (index)}
										<kbd
											class="inline-flex min-w-7 items-center justify-center rounded-fc-xs bg-fc-surface px-1.5 py-0.5 font-fc-body text-fc-xs font-medium whitespace-nowrap text-fc-fg"
											>{cap}</kbd
										>
									{/each}
								</span>
							{/each}
						</dd>
					</div>
				{/each}
			</dl>
		</SettingsSection>
	{/each}
</div>
