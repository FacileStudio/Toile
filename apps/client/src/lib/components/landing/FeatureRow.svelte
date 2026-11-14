<script lang="ts">
	import { Icon } from '@facile/muse';
	import type { Feature } from './content';

	let { feature }: { feature: Feature } = $props();
</script>

<!--
@component
One landing feature line: an icon plate beside its title and sentence.
-->

<div class="flex items-start gap-4">
	<span class="flex size-10 shrink-0 items-center justify-center rounded-fc-md bg-fc-component text-fc-fg">
		<Icon icon={feature.icon} size={20} />
	</span>
	<div class="flex min-w-0 flex-col gap-1">
		<h3 class="text-fc-md font-bold text-fc-fg">{feature.title}</h3>
		<p class="text-fc-sm text-fc-fg-muted">{feature.body}</p>
	</div>
</div>
