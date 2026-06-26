<script lang="ts">
  import type { Note, Camera } from "$lib/board.svelte";
  import { TEXT_COLOR } from "$lib/board.svelte";

  let {
    notes,
    camera,
    onJump,
    hidden = false,
  }: {
    notes: Note[];
    camera: Camera;
    onJump: (world: { x: number; y: number }) => void;
    hidden?: boolean;
  } = $props();

  const PAD = 8;
  const INNER_W = 190 - PAD * 2;
  const INNER_H = 132 - PAD * 2;

  let winW = $state(typeof window !== "undefined" ? window.innerWidth : 1280);
  let winH = $state(typeof window !== "undefined" ? window.innerHeight : 800);
  let dragging = $state(false);

  $effect(() => {
    const onResize = () => {
      winW = window.innerWidth;
      winH = window.innerHeight;
    };
    window.addEventListener("resize", onResize);
    return () => window.removeEventListener("resize", onResize);
  });

  const view = $derived({
    x: -camera.x / camera.scale,
    y: -camera.y / camera.scale,
    w: winW / camera.scale,
    h: winH / camera.scale,
  });

  const bbox = $derived.by(() => {
    let minX = Math.min(view.x, 0);
    let minY = Math.min(view.y, 0);
    let maxX = Math.max(view.x + view.w, 0);
    let maxY = Math.max(view.y + view.h, 0);
    for (const n of notes) {
      if (n.x < minX) minX = n.x;
      if (n.y < minY) minY = n.y;
      if (n.x + n.w > maxX) maxX = n.x + n.w;
      if (n.y + n.h > maxY) maxY = n.y + n.h;
    }
    const w = Math.max(maxX - minX, 1);
    const h = Math.max(maxY - minY, 1);
    const px = w * 0.08;
    const py = h * 0.08;
    return { x: minX - px, y: minY - py, w: w + px * 2, h: h + py * 2 };
  });

  const fit = $derived.by(() => {
    const s = Math.min(INNER_W / bbox.w, INNER_H / bbox.h);
    const offX = (INNER_W - bbox.w * s) / 2;
    const offY = (INNER_H - bbox.h * s) / 2;
    return { s, offX, offY };
  });

  function worldToMini(wx: number, wy: number): { x: number; y: number } {
    return {
      x: fit.offX + (wx - bbox.x) * fit.s,
      y: fit.offY + (wy - bbox.y) * fit.s,
    };
  }

  function miniToWorld(px: number, py: number): { x: number; y: number } {
    return {
      x: bbox.x + (px - fit.offX) / fit.s,
      y: bbox.y + (py - fit.offY) / fit.s,
    };
  }

  const viewRect = $derived.by(() => {
    const tl = worldToMini(view.x, view.y);
    return { x: tl.x, y: tl.y, w: view.w * fit.s, h: view.h * fit.s };
  });

  function jumpFromEvent(e: PointerEvent, svg: SVGSVGElement): void {
    const r = svg.getBoundingClientRect();
    const px = e.clientX - r.left;
    const py = e.clientY - r.top;
    onJump(miniToWorld(px, py));
  }

  function onPointerDown(e: PointerEvent): void {
    dragging = true;
    const svg = e.currentTarget as SVGSVGElement;
    svg.setPointerCapture(e.pointerId);
    jumpFromEvent(e, svg);
  }

  function onPointerMove(e: PointerEvent): void {
    if (!dragging) return;
    jumpFromEvent(e, e.currentTarget as SVGSVGElement);
  }

  function onPointerUp(e: PointerEvent): void {
    dragging = false;
    (e.currentTarget as SVGSVGElement).releasePointerCapture(e.pointerId);
  }
</script>

<div class="minimap liquid-glass" class:hidden class:dragging data-ui>
  <svg
    class="mini-svg"
    width={INNER_W}
    height={INNER_H}
    onpointerdown={onPointerDown}
    onpointermove={onPointerMove}
    onpointerup={onPointerUp}
    onpointercancel={onPointerUp}
    role="presentation"
  >
    {#each notes as n (n.id)}
      {@const p = worldToMini(n.x, n.y)}
      {#if n.color === TEXT_COLOR}
        <circle cx={p.x + Math.max(n.w * fit.s, 2) / 2} cy={p.y + Math.max(n.h * fit.s, 2) / 2} r="1.6" class="ink-dot" />
      {:else}
        <rect
          x={p.x}
          y={p.y}
          width={Math.max(n.w * fit.s, 2)}
          height={Math.max(n.h * fit.s, 2)}
          rx="1"
          fill={n.color}
          class="note-rect"
        />
      {/if}
    {/each}

    <rect
      class="view-rect"
      x={viewRect.x}
      y={viewRect.y}
      width={Math.max(viewRect.w, 3)}
      height={Math.max(viewRect.h, 3)}
      rx="2"
    />
  </svg>
</div>

<style>
  .minimap {
    position: fixed;
    top: 22px;
    right: 22px;
    z-index: 40;
    width: 190px;
    height: 132px;
    padding: 8px;
    border-radius: 16px;
    box-sizing: border-box;
    cursor: pointer;
    transition:
      opacity 0.3s var(--ease-soft),
      transform 0.3s var(--ease-soft);
  }
  .minimap.hidden {
    opacity: 0;
    transform: translateY(-12px);
    pointer-events: none;
  }
  .minimap.dragging {
    cursor: grabbing;
  }
  .mini-svg {
    display: block;
    touch-action: none;
    border-radius: 9px;
  }
  .note-rect {
    stroke: rgba(67, 65, 59, 0.12);
    stroke-width: 0.5px;
  }
  .ink-dot {
    fill: var(--ink-soft);
  }
  .view-rect {
    fill: rgba(67, 65, 59, 0.06);
    stroke: var(--ink);
    stroke-width: 1.5px;
  }

  @media (prefers-reduced-motion: reduce) {
    .minimap {
      transition: none !important;
    }
  }
</style>
