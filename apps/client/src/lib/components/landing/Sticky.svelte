<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { Box } from './hero';

	let {
		box,
		color,
		shown = true,
		node = $bindable(),
		children
	}: { box: Box; color: string; shown?: boolean; node?: HTMLElement; children: Snippet } = $props();
</script>

<!--
@component
One sticky note on the faux board. It stays mounted when hidden so it can pop in and out.
-->

<div
	bind:this={node}
	class="sticky flex flex-col gap-2 rounded-fc-sm p-3 text-fc-sm font-medium text-fc-fg shadow-sm"
	class:shown
	style:left="{box.x}px"
	style:top="{box.y}px"
	style:width="{box.w}px"
	style:height="{box.h}px"
	style:--paper={color}
>
	{@render children()}
</div>

<style>
	.sticky {
		position: absolute;
		background: color-mix(in oklch, var(--paper) 55%, var(--color-fc-page));
		opacity: 0;
		scale: 0.85;
		transition:
			opacity 0.35s ease,
			scale 0.45s cubic-bezier(0.34, 1.4, 0.64, 1);
	}
	.shown {
		opacity: 1;
		scale: 1;
	}
</style>
