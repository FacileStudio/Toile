<script lang="ts">
  import { scale } from "svelte/transition";

  let {
    x,
    y,
    onpaste,
    onclose,
  }: {
    x: number;
    y: number;
    onpaste: () => void;
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
  class="paste-menu"
  data-ui
  style="left:{x}px; top:{y}px"
  transition:scale={{ duration: 130, start: 0.9, opacity: 0 }}
>
  <button class="paste-btn liquid-glass" onclick={onpaste}>
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
      <rect x="8" y="2" width="8" height="4" rx="1" />
      <path
        d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"
      />
    </svg>
    Paste image
  </button>
</div>

<style>
  .ctx-backdrop {
    position: fixed;
    inset: 0;
    z-index: 90;
  }
  .paste-menu {
    position: fixed;
    z-index: 91;
    transform-origin: top left;
  }
  .paste-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    height: 38px;
    padding: 0 18px;
    border-radius: 999px;
    color: var(--ink);
    font-family: inherit;
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
    transition:
      background 0.16s ease,
      transform 0.16s var(--ease-soft);
  }
  .paste-btn:hover {
    background: rgba(40, 38, 32, 0.06);
  }
  .paste-btn:active {
    transform: scale(0.96);
  }
</style>
