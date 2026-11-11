<script lang="ts">
	let { x, y, color, name }: { x: number; y: number; color: string; name: string } = $props();
</script>

<!--
@component
A collaborator's live cursor on the faux board: a coloured arrow and a name tag. It travels on the
same curve as the kit's cursor.
-->

<span class="live" style:translate="{x}px {y}px" style:--who={color}>
	<svg width="16" height="20" viewBox="0 0 16 20" aria-hidden="true">
		<path d="M1 1l13.5 8.2-6.3 1.4L5 18.5Z" fill="var(--who)" />
	</svg>
	<span class="tag rounded-fc-pill px-2 py-0.5 text-fc-xs font-medium text-fc-fg">{name}</span>
</span>

<style>
	.live {
		position: absolute;
		top: 0;
		left: 0;
		z-index: 20;
		display: flex;
		align-items: flex-start;
		pointer-events: none;
		filter: drop-shadow(0 1px 2px rgb(0 0 0 / 0.18));
		transition: translate 0.9s cubic-bezier(0.45, 0, 0.1, 1);
	}
	.tag {
		margin-top: 14px;
		margin-left: -4px;
		white-space: nowrap;
		background: color-mix(in oklch, var(--who) 70%, var(--color-fc-page));
	}
</style>
