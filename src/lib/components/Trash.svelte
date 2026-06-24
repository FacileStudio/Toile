<script lang="ts">
  let {
    el = $bindable(),
    hot = false,
    armed = false,
  }: {
    el?: HTMLDivElement | null;
    hot?: boolean;
    armed?: boolean;
  } = $props();
</script>

<div
  bind:this={el}
  class="trash liquid-glass"
  class:hot
  class:armed
  data-ui
  aria-label="Drag a note here to delete"
>
  <svg
    viewBox="0 0 24 24"
    width="26"
    height="26"
    fill="none"
    stroke="currentColor"
    stroke-width="1.8"
    stroke-linecap="round"
    stroke-linejoin="round"
  >
    <path d="M19 6l-.8 13.1a2 2 0 0 1-2 1.9H7.8a2 2 0 0 1-2-1.9L5 6" />
    <path d="M10 11v6M14 11v6" />
    <g class="lid">
      <path d="M3 6h18" />
      <path d="M8 6V4a1 1 0 0 1 1-1h6a1 1 0 0 1 1 1v2" />
    </g>
  </svg>
</div>

<style>
  .trash {
    position: fixed;
    left: 22px;
    bottom: 22px;
    z-index: 40;
    width: 54px;
    height: 54px;
    display: grid;
    place-items: center;
    border-radius: 16px;
    color: var(--ink-soft);
    opacity: 0;
    transform: translateY(10px) scale(0.92);
    pointer-events: none;
    transition:
      transform 0.3s var(--ease-soft),
      opacity 0.3s var(--ease-soft),
      color 0.3s ease,
      background 0.3s ease,
      box-shadow 0.3s ease;
  }
  .trash.armed {
    opacity: 1;
    transform: scale(1.16);
    color: var(--ink);
  }
  .trash.hot {
    color: #e5484d;
    background: #ffeaea;
    border-color: rgba(229, 72, 77, 0.2);
    transform: scale(1.34) rotate(-4deg);
    box-shadow: 0 12px 30px rgba(40, 38, 32, 0.22);
  }
  .trash svg {
    overflow: visible;
  }
  .trash .lid {
    transition: transform 0.32s var(--ease-soft);
    transform-box: fill-box;
    transform-origin: 100% 100%;
  }
  .trash.armed .lid {
    transform: rotate(14deg);
  }
  .trash.hot .lid {
    transform: translateY(-1.5px) rotate(26deg);
  }
</style>
