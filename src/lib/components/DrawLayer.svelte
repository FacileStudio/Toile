<script lang="ts">
  import { board } from "../board.svelte";
  import { smoothPath, type Pt } from "../draw";

  let {
    live = [],
    liveColor = "#43413b",
    liveWidth = 4,
    marks = new Set<string>(),
  }: {
    live?: Pt[];
    liveColor?: string;
    liveWidth?: number;
    marks?: Set<string>;
  } = $props();

  const liveD = $derived(smoothPath(live));
</script>

<svg class="draw-layer" aria-hidden="true">
  {#each board.strokes as s (s.id)}
    <path
      data-stroke-id={s.id}
      d={s.d}
      stroke={s.color}
      stroke-width={s.width}
      fill="none"
      stroke-linecap="round"
      stroke-linejoin="round"
      opacity={marks.has(s.id) ? 0.25 : 1}
    />
  {/each}
  {#if live.length}
    <path
      d={liveD}
      stroke={liveColor}
      stroke-width={liveWidth}
      fill="none"
      stroke-linecap="round"
      stroke-linejoin="round"
    />
  {/if}
</svg>

<style>
  .draw-layer {
    position: absolute;
    top: 0;
    left: 0;
    width: 1px;
    height: 1px;
    overflow: visible;
    pointer-events: none;
  }
</style>
