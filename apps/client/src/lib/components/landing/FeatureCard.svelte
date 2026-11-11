<script lang="ts">
	import { Icon } from '@facile/muse';
	import type { Feature } from './content';

	let { feature, href }: { feature: Feature; href?: string } = $props();
</script>

<!--
@component
One landing card: an icon plate, a title and its sentence. With `href`, the title links out.
-->

<div class="flex h-full flex-col gap-3 rounded-fc-lg bg-fc-component p-6">
	<span class="flex size-10 items-center justify-center rounded-fc-md bg-fc-surface text-fc-fg">
		<Icon icon={feature.icon} size={20} />
	</span>
	<h3 class="text-fc-lg font-semibold text-fc-fg">
		{#if href}
			<a
				{href}
				class="rounded-fc-xs underline-offset-4 hover:underline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-fc-ring"
				>{feature.title}</a
			>
		{:else}
			{feature.title}
		{/if}
	</h3>
	<p class="text-fc-sm text-fc-fg-muted">{feature.body}</p>
</div>
