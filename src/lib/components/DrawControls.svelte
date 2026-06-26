<script lang="ts">
  let {
    color,
    width,
    oncolor,
    onwidth,
  }: {
    color: string;
    width: number;
    oncolor: (c: string) => void;
    onwidth: (w: number) => void;
  } = $props();

  const INKS = ["#43413b", "#c45b4c", "#3f7cc4", "#3f9b6a", "#9a6dd0", "#d39a2e"];
</script>

<div class="draw-ctl liquid-glass">
  <div class="inks">
    {#each INKS as c}
      <button
        class="ink"
        class:active={color === c}
        style="--c:{c}"
        title="Ink"
        aria-label="Ink color"
        onclick={() => oncolor(c)}
      ></button>
    {/each}
  </div>
  <span class="sep"></span>
  <input
    class="thick"
    type="range"
    min="1"
    max="28"
    step="1"
    value={width}
    aria-label="Stroke thickness"
    oninput={(e) => onwidth(+e.currentTarget.value)}
  />
  <span class="dot" style="width:{width}px; height:{width}px"></span>
</div>

<style>
  .draw-ctl {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 16px;
    border-radius: 999px;
  }
  .inks {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .ink {
    width: 26px;
    height: 26px;
    border-radius: 50%;
    border: none;
    background: var(--c);
    cursor: pointer;
    box-shadow: inset 0 0 0 1px rgba(40, 38, 32, 0.12);
    transition: transform 0.18s var(--ease-soft);
  }
  .ink:hover {
    transform: scale(1.12);
  }
  .ink.active {
    box-shadow:
      0 0 0 2px var(--paper),
      0 0 0 4px var(--ink);
  }
  .sep {
    width: 1px;
    height: 26px;
    background: rgba(40, 38, 32, 0.14);
  }
  .thick {
    width: 120px;
    accent-color: var(--ink);
    cursor: pointer;
  }
  .dot {
    flex: 0 0 auto;
    border-radius: 50%;
    background: var(--ink);
    min-width: 2px;
    min-height: 2px;
  }
</style>
