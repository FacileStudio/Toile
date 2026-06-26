<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    onclick,
    title = "",
    kbd = "",
    active = false,
    disabled = false,
    children,
  }: {
    onclick: () => void;
    title?: string;
    kbd?: string;
    active?: boolean;
    disabled?: boolean;
    children?: Snippet;
  } = $props();
</script>

<button
  class="circle liquid-glass"
  class:active
  {onclick}
  {disabled}
  aria-label={title}
>
  {@render children?.()}
  {#if title}
    <span class="tip">{title}{#if kbd}<kbd>{kbd}</kbd>{/if}</span>
  {/if}
</button>

<style>
  .circle {
    width: 54px;
    height: 54px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    color: var(--ink);
    cursor: pointer;
    font-family: inherit;
    transition:
      transform 0.18s var(--ease-soft),
      opacity 0.18s var(--ease-soft);
  }
  .circle:hover {
    transform: scale(1.06);
  }
  .circle:active {
    transform: scale(0.92);
  }
  .circle.active {
    color: var(--ink);
    box-shadow:
      0 0 0 2px var(--ink) inset,
      0 8px 30px rgba(40, 38, 32, 0.16);
  }
  .circle:disabled {
    opacity: 0.35;
    cursor: default;
    transform: none;
  }
  .circle :global(svg) {
    display: block;
  }
</style>
