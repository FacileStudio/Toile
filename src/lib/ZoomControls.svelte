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
  <button class="znav liquid-glass" onclick={onZoomOut} aria-label="Zoom out"
    >−</button
  >
  <button class="zinfo liquid-glass" onclick={onReset} title="Reset to 100%">
    <span class="zinner"
      ><span class="zval">{zoom}</span><span class="zpct">%</span></span
    >
  </button>
  <button class="znav liquid-glass" onclick={onZoomIn} aria-label="Zoom in"
    >+</button
  >
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
    font-size: 24px;
    font-weight: 500;
    display: grid;
    place-items: center;
    line-height: 1;
  }
  .zinfo {
    height: 54px;
    border-radius: 999px;
    padding: 0 18px;
    min-width: 70px;
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
    transform: translateY(-2px);
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
