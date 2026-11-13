<script lang="ts">
	let { seed, thumbnail }: { seed: string; thumbnail: string | null } = $props();

	let broken = $state(false);

	/** A small deterministic pseudo-random sequence from the board id, so each board keeps its own sketch. */
	function sequence(text: string): () => number {
		let state = 2166136261;
		for (const char of text) state = Math.imul(state ^ char.charCodeAt(0), 16777619);
		return () => {
			state = Math.imul(state ^ (state >>> 15), 2246822507);
			state = Math.imul(state ^ (state >>> 13), 3266489909);
			return ((state ^= state >>> 16) >>> 0) / 4294967296;
		};
	}

	const notes = $derived.by(() => {
		const next = sequence(seed);
		const count = 3 + Math.floor(next() * 3);
		return Array.from({ length: count }, (_, i) => ({
			left: 6 + (i * 68) / count + next() * 6,
			top: 10 + next() * 42,
			tilt: (next() - 0.5) * 10,
			drift: (next() - 0.5) * 14
		}));
	});
</script>

<div class="relative aspect-[16/10] overflow-hidden rounded-fc-md bg-fc-surface" aria-hidden="true">
	{#if thumbnail && !broken}
		<img
			src={thumbnail}
			alt=""
			loading="lazy"
			class="size-full object-cover transition-[scale] duration-500 group-hover:scale-105 motion-reduce:transition-none"
			onerror={() => (broken = true)}
		/>
	{:else}
		<div class="preview-dots absolute inset-0"></div>
		{#each notes as note, i (i)}
			<span
				class="preview-note absolute aspect-square w-[22%] rounded-fc-xs bg-fc-component shadow-xs"
				style:left="{note.left}%"
				style:top="{note.top}%"
				style:--tilt="{note.tilt}deg"
				style:--drift="{note.drift}px"
				style:--i={i}
			></span>
		{/each}
	{/if}
</div>

<style>
	.preview-dots {
		background-image: radial-gradient(circle, var(--color-fc-component) 1.2px, transparent 1.4px);
		background-size: 14px 14px;
	}

	.preview-note {
		rotate: var(--tilt);
		transition:
			rotate 0.5s cubic-bezier(0.22, 1, 0.36, 1),
			translate 0.5s cubic-bezier(0.22, 1, 0.36, 1);
		transition-delay: calc(var(--i) * 25ms);
	}

	:global(.group:hover) .preview-note,
	:global(.group:focus-visible) .preview-note {
		rotate: calc(var(--tilt) * -1);
		translate: var(--drift) -6px;
	}

	@media (prefers-reduced-motion: reduce) {
		.preview-note {
			transition: none;
		}
	}
</style>
