<script lang="ts">
  import { scale } from "svelte/transition";

  let {
    x,
    y,
    colors,
    showColors = true,
    oncolor,
    ondelete,
    onclose,
  }: {
    x: number;
    y: number;
    colors: string[];
    showColors?: boolean;
    oncolor: (color: string) => void;
    ondelete: () => void;
    onclose: () => void;
  } = $props();
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="ctx-backdrop"
  data-ui
  onpointerdown={onclose}
  oncontextmenu={(e) => {
    e.preventDefault();
    onclose();
  }}
></div>
<div
  class="ctx-menu"
  data-ui
  style="left:{x}px; top:{y}px"
  transition:scale={{ duration: 130, start: 0.9, opacity: 0 }}
>
  {#if showColors}
    <div class="ctx-colors liquid-glass">
      {#each colors as color}
        <button
          class="ctx-swatch"
          style="--c:{color}"
          aria-label="Set color"
          onclick={() => oncolor(color)}
        ></button>
      {/each}
    </div>
  {/if}
  <button class="ctx-delete liquid-glass" onclick={ondelete}>
    <svg
      viewBox="0 0 24 24"
      width="16"
      height="16"
      fill="none"
      stroke="currentColor"
      stroke-width="1.8"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <path d="M19 6l-.8 13.1a2 2 0 0 1-2 1.9H7.8a2 2 0 0 1-2-1.9L5 6" />
      <path d="M10 11v6M14 11v6" />
      <path d="M3 6h18" />
      <path d="M8 6V4a1 1 0 0 1 1-1h6a1 1 0 0 1 1 1v2" />
    </svg>
    Delete
  </button>
</div>

<style>
  .ctx-backdrop {
    position: fixed;
    inset: 0;
    z-index: 90;
  }
  .ctx-menu {
    position: fixed;
    z-index: 91;
    transform-origin: top left;
    display: flex;
    flex-direction: column;
    align-items: stretch;
    gap: 8px;
  }
  .ctx-colors {
    display: flex;
    justify-content: center;
    gap: 8px;
    padding: 8px 10px;
    border-radius: 999px;
  }
  .ctx-swatch {
    width: 24px;
    height: 24px;
    border-radius: 50%;
    border: none;
    background: var(--c);
    cursor: pointer;
    box-shadow:
      inset 0 0 0 1px rgba(40, 38, 32, 0.08),
      0 1px 3px rgba(40, 38, 32, 0.14);
    transition: transform 0.16s var(--ease-soft);
  }
  .ctx-swatch:hover {
    transform: scale(1.18);
  }
  .ctx-delete {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 7px;
    height: 38px;
    border-radius: 999px;
    color: #e5484d;
    font-family: inherit;
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
    transition:
      background 0.16s ease,
      transform 0.16s var(--ease-soft);
  }
  .ctx-delete:hover {
    background: rgba(229, 72, 77, 0.12);
  }
  .ctx-delete:active {
    transform: scale(0.96);
  }
</style>
