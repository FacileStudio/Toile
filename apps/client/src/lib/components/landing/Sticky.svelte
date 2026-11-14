<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { Box, Spot } from './hero';

	let {
		box,
		color,
		shown = true,
		shift = { x: 0, y: 0 },
		selected = null,
		own = false,
		node = $bindable(),
		children
	}: {
		box: Box;
		color: string;
		shown?: boolean;
		shift?: Spot;
		selected?: string | null;
		own?: boolean;
		node?: HTMLElement;
		children: Snippet;
	} = $props();

	const CORNERS = [
		[0, 0],
		[100, 0],
		[0, 100],
		[100, 100]
	] as const;
</script>

<!--
@component
One sticky note on the faux board. It stays mounted when hidden so it can pop in and out. `shift`
slides it on the cursor's travel curve, `selected` draws a person's selection outline in their
colour, and `own` adds the visitor's corner handles as the app's SelectionHandles draws them.
-->

<div
	bind:this={node}
	class="sticky flex flex-col gap-2 rounded-fc-sm p-3 text-fc-sm font-medium text-fc-fg shadow-sm"
	class:shown
	class:picked={!!selected}
	style:left="{box.x}px"
	style:top="{box.y}px"
	style:width="{box.w}px"
	style:height="{box.h}px"
	style:translate="{shift.x}px {shift.y}px"
	style:--paper={color}
	style:--pick={selected ?? 'transparent'}
>
	{@render children()}
	{#each CORNERS as [x, y] (`${x}-${y}`)}
		<span
			class="handle absolute size-3 -translate-x-1/2 -translate-y-1/2 rounded-[3px] bg-fc-info shadow-sm"
			class:on={own && !!selected}
			style:left="{x}%"
			style:top="{y}%"
		></span>
	{/each}
</div>

<style>
	.sticky {
		position: absolute;
		background: color-mix(in oklch, var(--paper) 55%, var(--color-fc-page));
		opacity: 0;
		scale: 0.85;
		outline: 2px solid transparent;
		transition:
			opacity 0.35s ease,
			scale 0.45s cubic-bezier(0.34, 1.4, 0.64, 1),
			translate 0.9s cubic-bezier(0.45, 0, 0.1, 1),
			outline-color 0.2s ease;
	}
	.shown {
		opacity: 1;
		scale: 1;
	}
	.picked {
		outline-color: var(--pick);
	}
	.handle {
		opacity: 0;
		transition: opacity 0.2s ease;
	}
	.on {
		opacity: 1;
	}
</style>
