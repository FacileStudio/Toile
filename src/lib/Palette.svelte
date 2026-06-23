<script lang="ts">
  let {
    colors,
    onpick,
  }: { colors: string[]; onpick: (color: string) => void } = $props();
</script>

<div class="palette liquid-glass">
  {#each colors as color, i}
    <button
      class="swatch"
      style="--c:{color}; animation-delay:{i * 40}ms"
      title="Add note"
      aria-label="Add note"
      onclick={() => onpick(color)}
    ></button>
  {/each}
</div>

<style>
  .palette {
    display: flex;
    gap: 12px;
    padding: 12px 16px;
    border-radius: 999px;
  }
  .swatch {
    width: 30px;
    height: 30px;
    border-radius: 50%;
    border: none;
    background: var(--c);
    cursor: pointer;
    box-shadow:
      inset 0 0 0 1px rgba(40, 38, 32, 0.06),
      0 2px 5px rgba(40, 38, 32, 0.12);
    transition:
      transform 0.22s var(--ease-soft),
      box-shadow 0.22s var(--ease-soft);
    animation: pop 0.4s var(--ease-soft) both;
  }
  .swatch:hover {
    transform: translateY(-4px) scale(1.12);
    box-shadow:
      inset 0 0 0 1px rgba(40, 38, 32, 0.06),
      0 6px 14px rgba(40, 38, 32, 0.2);
  }
  .swatch:active {
    transform: translateY(-1px) scale(0.96);
  }

  @keyframes pop {
    from {
      opacity: 0;
      transform: translateY(8px) scale(0.6);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .swatch {
      animation: none !important;
      transition: none !important;
    }
  }
</style>
