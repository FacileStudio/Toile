<script lang="ts">
  let {
    zoom,
    onZoomIn,
    onZoomOut,
    onReset,
  }: {
    zoom: number;
    onZoomIn: () => void;
    onZoomOut: () => void;
    onReset: () => void;
  } = $props();
</script>

<div class="zoom" data-ui>
  <button class="znav liquid-glass" onclick={onZoomOut} aria-label="Zoom out">
    <svg
      viewBox="0 0 24 24"
      width="22"
      height="22"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
    >
      <line x1="6" y1="12" x2="18" y2="12" />
    </svg>
  </button>
  <button class="zinfo liquid-glass" onclick={onReset} title="Reset to 100%">
    <span class="zinner"
      ><span class="zval">{zoom}</span><span class="zpct">%</span></span
    >
  </button>
  <button class="znav liquid-glass" onclick={onZoomIn} aria-label="Zoom in">
    <svg
      viewBox="0 0 24 24"
      width="22"
      height="22"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
    >
      <line x1="12" y1="6" x2="12" y2="18" />
      <line x1="6" y1="12" x2="18" y2="12" />
    </svg>
  </button>
</div>

<style>
  .zoom {
    position: fixed;
    right: 22px;
    bottom: 24px;
    z-index: 40;
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .znav,
  .zinfo {
    color: var(--ink);
    cursor: pointer;
    font-family: inherit;
    transition:
      transform 0.18s var(--ease-soft),
      box-shadow 0.18s ease;
  }
  .znav {
    width: 54px;
    height: 54px;
    border-radius: 50%;
    display: grid;
    place-items: center;
  }
  .znav svg {
    display: block;
  }
  .zinfo {
    height: 54px;
    width: 84px;
    border-radius: 999px;
    display: grid;
    place-items: center;
    white-space: nowrap;
    font-size: 17px;
    font-weight: 650;
    font-variant-numeric: tabular-nums;
    font-feature-settings: "tnum" 1;
    line-height: 1;
  }
  .zinner {
    display: inline-flex;
    align-items: baseline;
    transform: translateX(2px);
  }
  .zpct {
    margin-left: 1px;
    font-size: 13px;
    font-weight: 600;
    color: var(--ink-soft);
  }
  .znav:hover,
  .zinfo:hover {
    transform: scale(1.06);
  }
  .znav:active,
  .zinfo:active {
    transform: scale(0.92);
  }

  @media (prefers-reduced-motion: reduce) {
    .znav,
    .zinfo {
      transition: none !important;
    }
  }
</style>
