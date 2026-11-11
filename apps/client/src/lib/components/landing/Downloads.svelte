<script lang="ts">
	import { Icon } from '@facile/muse';
	import { DOWNLOADS, downloadUrl } from '$lib/downloads';
</script>

<!--
@component
The installers of the latest release, one tile per asset. Each tile is a plain download link.
-->

<ul class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
	{#each DOWNLOADS as item (item.asset)}
		<li>
			<a
				href={downloadUrl(item.asset)}
				class="flex min-h-16 items-center gap-3 rounded-fc-lg bg-fc-component px-4 py-3 transition-colors hover:bg-fc-surface focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-fc-ring"
			>
				<span class="flex size-10 shrink-0 items-center justify-center rounded-fc-md bg-fc-surface text-fc-fg">
					<Icon icon={item.icon} size={20} />
				</span>
				<span class="flex min-w-0 flex-col">
					<span class="text-fc-sm font-semibold text-fc-fg">{item.label}</span>
					<span class="text-fc-xs text-fc-fg-muted">{item.detail}</span>
				</span>
			</a>
		</li>
	{/each}
</ul>
