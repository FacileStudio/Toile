<script lang="ts">
  import { onMount } from "svelte";
  import { fade, scale } from "svelte/transition";
  import { listen } from "@tauri-apps/api/event";
  import Postit from "$lib/Postit.svelte";
  import {
    board,
    COLORS,
    MIN_SCALE,
    MAX_SCALE,
    type Note,
  } from "$lib/board.svelte";

  const GRID = 26;

  let trashEl = $state<HTMLDivElement | null>(null);
  let editingId = $state<string | null>(null);
  let dragId = $state<string | null>(null);
  let trashHot = $state(false);
  let panning = $state(false);

  let menu = $state<{ x: number; y: number; id: string } | null>(null);

  let dragNote: Note | null = null;
  let last = { x: 0, y: 0 };
  let raf: number | null = null;

  const lastSig = new Map<string, string>();
  let writeTimer: ReturnType<typeof setTimeout>;

  const clamp = (v: number, lo: number, hi: number) =>
    Math.min(hi, Math.max(lo, v));

  function toWorld(cx: number, cy: number) {
    return {
      x: (cx - board.camera.x) / board.camera.scale,
      y: (cy - board.camera.y) / board.camera.scale,
    };
  }

  function viewCenterWorld() {
    return toWorld(window.innerWidth / 2, window.innerHeight / 2);
  }

  function animateCamera(target: { x: number; y: number; scale: number }) {
    if (raf) cancelAnimationFrame(raf);
    const start = { ...board.camera };
    const t0 = performance.now();
    const dur = 460;
    const ease = (t: number) => 1 - Math.pow(1 - t, 3);
    const step = (now: number) => {
      const p = Math.min(1, (now - t0) / dur);
      const k = ease(p);
      board.camera = {
        x: start.x + (target.x - start.x) * k,
        y: start.y + (target.y - start.y) * k,
        scale: start.scale + (target.scale - start.scale) * k,
      };
      raf = p < 1 ? requestAnimationFrame(step) : null;
    };
    raf = requestAnimationFrame(step);
  }

  function stopTween() {
    if (raf) cancelAnimationFrame(raf);
    raf = null;
  }

  function addNote(color: string) {
    commitEdit();
    const c = viewCenterWorld();
    const note = board.add(color, c.x, c.y);
    focusNote(note);
  }

  function focusNote(note: Note) {
    editingId = note.id;
    const targetScale = clamp(
      Math.max(board.camera.scale, 1.3),
      MIN_SCALE,
      MAX_SCALE,
    );
    const cx = note.x + note.w / 2;
    const cy = note.y + note.h / 2;
    animateCamera({
      x: window.innerWidth / 2 - cx * targetScale,
      y: window.innerHeight * 0.44 - cy * targetScale,
      scale: targetScale,
    });
  }

  function commitEdit() {
    editingId = null;
  }

  function resetView() {
    commitEdit();
    animateCamera({ x: 0, y: 0, scale: 1 });
  }

  function zoomBy(factor: number) {
    const cx = window.innerWidth / 2;
    const cy = window.innerHeight / 2;
    const w = toWorld(cx, cy);
    const ns = clamp(board.camera.scale * factor, MIN_SCALE, MAX_SCALE);
    animateCamera({ x: cx - w.x * ns, y: cy - w.y * ns, scale: ns });
  }

  function onWheel(e: WheelEvent) {
    e.preventDefault();
    stopTween();
    menu = null;
    if (e.ctrlKey || e.metaKey) {
      const ns = clamp(
        board.camera.scale * Math.exp(-e.deltaY * 0.01),
        MIN_SCALE,
        MAX_SCALE,
      );
      const w = toWorld(e.clientX, e.clientY);
      board.camera = { x: e.clientX - w.x * ns, y: e.clientY - w.y * ns, scale: ns };
    } else {
      board.camera = {
        ...board.camera,
        x: board.camera.x - e.deltaX,
        y: board.camera.y - e.deltaY,
      };
    }
  }

  function onContextMenu(e: MouseEvent) {
    e.preventDefault();
    const noteEl = (e.target as HTMLElement).closest(
      "[data-note]",
    ) as HTMLElement | null;
    if (!noteEl) {
      menu = null;
      return;
    }
    const id = noteEl.dataset.note!;
    const note = board.notes.find((n) => n.id === id);
    if (!note) return;
    board.bringToFront(note);
    const mw = 232;
    const mh = 104;
    menu = {
      x: Math.min(e.clientX, window.innerWidth - mw - 8),
      y: Math.min(e.clientY, window.innerHeight - mh - 8),
      id,
    };
  }

  function setColor(id: string, color: string) {
    const note = board.notes.find((n) => n.id === id);
    if (note) note.color = color;
    menu = null;
  }

  function deleteFromMenu(id: string) {
    if (editingId === id) editingId = null;
    board.remove(id);
    menu = null;
  }

  function onPointerDown(e: PointerEvent) {
    if (e.button !== 0) return;
    menu = null;
    const target = e.target as HTMLElement;
    const noteEl = target.closest("[data-note]") as HTMLElement | null;

    if (noteEl) {
      const id = noteEl.dataset.note!;
      if (id === editingId) return;
      commitEdit();
      const note = board.notes.find((n) => n.id === id);
      if (!note) return;
      board.bringToFront(note);
      dragId = id;
      dragNote = note;
      last = { x: e.clientX, y: e.clientY };
    } else if (!target.closest("[data-ui]")) {
      commitEdit();
      stopTween();
      panning = true;
      last = { x: e.clientX, y: e.clientY };
    }
  }

  function onPointerMove(e: PointerEvent) {
    // recover if the button was released outside the window
    if ((dragId || panning) && e.buttons === 0) {
      endInteraction();
      return;
    }
    if (dragId && dragNote) {
      dragNote.x += (e.clientX - last.x) / board.camera.scale;
      dragNote.y += (e.clientY - last.y) / board.camera.scale;
      last = { x: e.clientX, y: e.clientY };
      trashHot = overTrash(e.clientX, e.clientY);
    } else if (panning) {
      board.camera = {
        ...board.camera,
        x: board.camera.x + (e.clientX - last.x),
        y: board.camera.y + (e.clientY - last.y),
      };
      last = { x: e.clientX, y: e.clientY };
    }
  }

  function endInteraction() {
    dragId = null;
    dragNote = null;
    trashHot = false;
    panning = false;
  }

  function onPointerUp() {
    if (dragId && trashHot) board.remove(dragId);
    endInteraction();
  }

  function overTrash(cx: number, cy: number): boolean {
    if (!trashEl) return false;
    const r = trashEl.getBoundingClientRect();
    const pad = 22;
    return (
      cx >= r.left - pad &&
      cx <= r.right + pad &&
      cy >= r.top - pad &&
      cy <= r.bottom + pad
    );
  }

  function onDblClick(e: MouseEvent) {
    const noteEl = (e.target as HTMLElement).closest(
      "[data-note]",
    ) as HTMLElement | null;
    if (!noteEl) return;
    const note = board.notes.find((n) => n.id === noteEl.dataset.note);
    if (note) {
      board.bringToFront(note);
      focusNote(note);
    }
  }

  function onKeyDown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      commitEdit();
      menu = null;
    }
  }

  // ---- live file sync ----
  onMount(() => {
    let unlisten: Array<() => void> = [];
    (async () => {
      await board.init();
      for (const n of board.notes) lastSig.set(n.id, JSON.stringify(n));

      unlisten.push(
        await listen<Note>("note-changed", (e) => {
          const inc = e.payload;
          if (inc.id === editingId || inc.id === dragId) return;
          const existing = board.notes.find((n) => n.id === inc.id);
          if (existing) {
            existing.x = inc.x;
            existing.y = inc.y;
            existing.w = inc.w;
            existing.h = inc.h;
            existing.color = inc.color;
            existing.text = inc.text;
            existing.z = inc.z;
            board.noteZTop(inc.z);
            lastSig.set(inc.id, JSON.stringify(existing));
          } else {
            board.notes.push(inc);
            board.noteZTop(inc.z);
            lastSig.set(inc.id, JSON.stringify(inc));
          }
        }),
      );

      unlisten.push(
        await listen<{ id: string }>("note-removed", (e) => {
          const id = e.payload.id;
          if (id === editingId) editingId = null;
          board.notes = board.notes.filter((n) => n.id !== id);
          lastSig.delete(id);
        }),
      );
    })();
    return () => unlisten.forEach((u) => u());
  });

  // ---- persist changed notes to disk (debounced diff) ----
  $effect(() => {
    const snap = board.notes.map(
      (n) => [n.id, JSON.stringify(n)] as [string, string],
    );
    clearTimeout(writeTimer);
    writeTimer = setTimeout(() => {
      for (const [id, sig] of snap) {
        if (lastSig.get(id) !== sig) {
          lastSig.set(id, sig);
          board.writeNote(id);
        }
      }
      board.saveCamera();
    }, 300);
  });

  const gridStyle = $derived(
    `background-position:${board.camera.x}px ${board.camera.y}px;` +
      `background-size:${GRID * board.camera.scale}px ${GRID * board.camera.scale}px;`,
  );
  const worldStyle = $derived(
    `transform:translate(${board.camera.x}px,${board.camera.y}px) scale(${board.camera.scale});`,
  );
  const zoomPct = $derived(Math.round(board.camera.scale * 100));
</script>

<svelte:window
  onpointermove={onPointerMove}
  onpointerup={onPointerUp}
  onpointercancel={endInteraction}
  onkeydown={onKeyDown}
/>

<div class="titlebar" data-ui data-tauri-drag-region></div>

<main
  class="viewport"
  class:panning
  style={gridStyle}
  onwheel={onWheel}
  onpointerdown={onPointerDown}
  ondblclick={onDblClick}
  oncontextmenu={onContextMenu}
>
  <div class="world" style={worldStyle}>
    {#each board.notes as note (note.id)}
      <Postit
        {note}
        editing={editingId === note.id}
        dragging={dragId === note.id}
        doomed={dragId === note.id && trashHot}
      />
    {/each}
  </div>

  {#if board.notes.length === 0}
    <div class="hint" transition:fade={{ duration: 300 }}>
      <div class="hint-title">Tableau</div>
      <div class="hint-sub">Pick a color below to drop your first note</div>
    </div>
  {/if}

  <div
    bind:this={trashEl}
    class="trash"
    class:hot={trashHot}
    class:armed={dragId !== null}
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

  <div class="palette" data-ui transition:fade={{ duration: 200 }}>
    {#each COLORS as color, i}
      <button
        class="swatch"
        style="--c:{color}; animation-delay:{i * 40}ms"
        title="Add note"
        aria-label="Add note"
        onclick={() => addNote(color)}
      ></button>
    {/each}
  </div>

  <div class="zoom" data-ui>
    <button class="znav" onclick={() => zoomBy(1 / 1.25)} aria-label="Zoom out">−</button>
    <button class="zinfo" onclick={resetView} title="Reset view">{zoomPct}%</button>
    <button class="znav" onclick={() => zoomBy(1.25)} aria-label="Zoom in">+</button>
  </div>
</main>

{#if menu}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="ctx-backdrop"
    data-ui
    onpointerdown={() => (menu = null)}
    oncontextmenu={(e) => {
      e.preventDefault();
      menu = null;
    }}
  ></div>
  <div
    class="ctx-menu"
    data-ui
    style="left:{menu.x}px; top:{menu.y}px"
    transition:scale={{ duration: 130, start: 0.9, opacity: 0 }}
  >
    <div class="ctx-colors">
      {#each COLORS as color}
        <button
          class="ctx-swatch"
          style="--c:{color}"
          aria-label="Set color"
          onclick={() => setColor(menu!.id, color)}
        ></button>
      {/each}
    </div>
    <button class="ctx-delete" onclick={() => deleteFromMenu(menu!.id)}>
      <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
        <path d="M19 6l-.8 13.1a2 2 0 0 1-2 1.9H7.8a2 2 0 0 1-2-1.9L5 6" />
        <path d="M10 11v6M14 11v6" />
        <path d="M3 6h18" />
        <path d="M8 6V4a1 1 0 0 1 1-1h6a1 1 0 0 1 1 1v2" />
      </svg>
      Delete
    </button>
  </div>
{/if}

<style>
  .titlebar {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    height: 30px;
    z-index: 50;
  }

  .viewport {
    position: fixed;
    inset: 0;
    overflow: hidden;
    background-color: var(--paper);
    background-image: radial-gradient(circle, var(--dot) 1.4px, transparent 1.6px);
    cursor: default;
  }
  .viewport.panning {
    cursor: grabbing;
  }
  .world {
    position: absolute;
    top: 0;
    left: 0;
    transform-origin: 0 0;
  }

  .hint {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    pointer-events: none;
    text-align: center;
  }
  .hint-title {
    font-size: 40px;
    font-weight: 700;
    letter-spacing: -0.02em;
    color: rgba(67, 65, 59, 0.32);
  }
  .hint-sub {
    font-size: 15px;
    font-weight: 500;
    color: rgba(67, 65, 59, 0.28);
  }

  .trash {
    position: fixed;
    left: 22px;
    bottom: 22px;
    width: 54px;
    height: 54px;
    display: grid;
    place-items: center;
    border-radius: 16px;
    color: var(--ink-soft);
    background: rgba(255, 255, 255, 0.7);
    backdrop-filter: blur(8px);
    border: 1px solid rgba(40, 38, 32, 0.08);
    box-shadow: 0 4px 16px rgba(40, 38, 32, 0.08);
    opacity: 0.55;
    transition:
      transform 0.3s var(--ease-soft),
      opacity 0.3s var(--ease-soft),
      color 0.3s ease,
      background 0.3s ease,
      box-shadow 0.3s ease;
  }
  .trash.armed {
    opacity: 1;
    transform: scale(1.07);
    color: var(--ink);
    box-shadow: 0 6px 20px rgba(40, 38, 32, 0.16);
  }
  .trash.hot {
    color: #e5484d;
    background: #ffeaea;
    border-color: rgba(229, 72, 77, 0.4);
    transform: scale(1.18) rotate(-4deg);
    box-shadow: 0 8px 24px rgba(229, 72, 77, 0.28);
  }
  .trash .lid {
    transition: transform 0.32s var(--ease-soft);
    transform-box: fill-box;
    transform-origin: 86% 95%;
  }
  .trash.armed .lid {
    transform: translateY(-1.2px) rotate(-16deg);
  }
  .trash.hot .lid {
    transform: translateY(-2.5px) rotate(-26deg);
  }

  .palette {
    position: fixed;
    left: 50%;
    bottom: 24px;
    transform: translateX(-50%);
    display: flex;
    gap: 12px;
    padding: 12px 16px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.72);
    backdrop-filter: blur(12px);
    border: 1px solid rgba(40, 38, 32, 0.07);
    box-shadow: 0 6px 24px rgba(40, 38, 32, 0.1);
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

  /* zoom controls: circle nav buttons + center pill, matched to palette height */
  .zoom {
    position: fixed;
    right: 22px;
    bottom: 24px;
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .znav,
  .zinfo {
    background: rgba(255, 255, 255, 0.72);
    backdrop-filter: blur(12px);
    border: 1px solid rgba(40, 38, 32, 0.07);
    box-shadow: 0 6px 24px rgba(40, 38, 32, 0.1);
    color: var(--ink);
    cursor: pointer;
    font-family: inherit;
    transition:
      transform 0.18s var(--ease-soft),
      background 0.18s ease;
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
    padding: 0 20px;
    font-size: 14px;
    font-weight: 500;
    color: var(--ink-soft);
    min-width: 64px;
  }
  .znav:hover,
  .zinfo:hover {
    background: rgba(255, 255, 255, 0.95);
  }
  .znav:active,
  .zinfo:active {
    transform: scale(0.92);
  }

  /* right-click context menu */
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
    background: rgba(255, 255, 255, 0.85);
    backdrop-filter: blur(16px);
    border: 1px solid rgba(40, 38, 32, 0.08);
    box-shadow: 0 10px 34px rgba(40, 38, 32, 0.2);
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
    border: 1px solid rgba(229, 72, 77, 0.18);
    border-radius: 999px;
    background: rgba(255, 235, 235, 0.9);
    backdrop-filter: blur(16px);
    color: #e5484d;
    font-family: inherit;
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
    box-shadow: 0 8px 28px rgba(229, 72, 77, 0.18);
    transition:
      background 0.16s ease,
      transform 0.16s var(--ease-soft);
  }
  .ctx-delete:hover {
    background: #e5484d;
    color: #fff;
  }
  .ctx-delete:active {
    transform: scale(0.96);
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
    *,
    .swatch {
      animation: none !important;
      transition: none !important;
    }
  }
</style>
