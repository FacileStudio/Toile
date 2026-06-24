<script lang="ts">
  import { onMount } from "svelte";
  import { fade } from "svelte/transition";
  import { listen } from "@tauri-apps/api/event";
  import { readImage } from "@tauri-apps/plugin-clipboard-manager";
  import { openPath, openUrl } from "@tauri-apps/plugin-opener";
  import type { Image } from "@tauri-apps/api/image";
  import Postit from "$lib/components/Postit.svelte";
  import Palette from "$lib/components/Palette.svelte";
  import AttachButton from "$lib/components/AttachButton.svelte";
  import ZoomControls from "$lib/components/ZoomControls.svelte";
  import Trash from "$lib/components/Trash.svelte";
  import ContextMenu from "$lib/components/ContextMenu.svelte";
  import PasteMenu from "$lib/components/PasteMenu.svelte";
  import Lightbox from "$lib/components/Lightbox.svelte";
  import { board, COLORS, MIN_SCALE, MAX_SCALE, type Note } from "$lib/board.svelte";
  import { assetKind, assetPath, isAssetOnly } from "$lib/assets";

  const GRID = 26;

  let trashEl = $state<HTMLDivElement | null>(null);
  let editingId = $state<string | null>(null);
  let dragId = $state<string | null>(null);
  let trashHot = $state(false);
  let panning = $state(false);

  let menu = $state<{ x: number; y: number; id: string } | null>(null);
  let pasteMenu = $state<{
    x: number;
    y: number;
    wx: number;
    wy: number;
  } | null>(null);
  let pasteImage: Image | null = null;
  let lightbox = $state<string | null>(null);
  let lightboxKind = $state<"image" | "video">("image");
  let lightboxOrigin = $state<DOMRect | null>(null);
  let lightboxSourceEl: HTMLElement | null = null;

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
    const cx = window.innerWidth / 2;
    const cy = window.innerHeight / 2;
    const w = toWorld(cx, cy);
    animateCamera({ x: cx - w.x, y: cy - w.y, scale: 1 });
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
    pasteMenu = null;
    if (e.ctrlKey || e.metaKey) {
      const ns = clamp(
        board.camera.scale * Math.exp(-e.deltaY * 0.01),
        MIN_SCALE,
        MAX_SCALE,
      );
      const w = toWorld(e.clientX, e.clientY);
      board.camera = {
        x: e.clientX - w.x * ns,
        y: e.clientY - w.y * ns,
        scale: ns,
      };
    } else {
      board.camera = {
        ...board.camera,
        x: board.camera.x - e.deltaX,
        y: board.camera.y - e.deltaY,
      };
    }
  }

  // ---- assets: paste / drop / pick -> save bytes -> markdown ref in a note ----
  // Media embeds inline (`![name](…)`); anything else becomes a click-to-open
  // file link (`[name](…)`). The original filename rides along so audio/video
  // tiles can label themselves. Backend just hashes bytes — type-agnostic.
  const randomColor = () => COLORS[Math.floor(Math.random() * COLORS.length)];

  function extOf(file: File): string {
    const dot = file.name.lastIndexOf(".");
    if (dot > 0) return file.name.slice(dot + 1);
    return file.type.split("/")[1] ?? "bin";
  }

  async function storeAsset(file: File): Promise<string> {
    const bytes = new Uint8Array(await file.arrayBuffer());
    const path = await board.saveAsset(bytes, extOf(file));
    const name = (file.name || path.split("/").pop() || "file").replace(
      /[\[\]\n]/g,
      " ",
    );
    return assetKind(path) === "file" ? `[${name}](${path})` : `![${name}](${path})`;
  }

  function appendAssets(note: Note, md: string) {
    note.text = note.text ? `${note.text}\n${md}` : md;
  }

  async function addAssetNotes(files: File[]) {
    if (!files.length) return;
    const refs = await Promise.all(files.map(storeAsset));
    commitEdit();
    const c = viewCenterWorld();
    refs.forEach((md, i) =>
      board.add(randomColor(), c.x + i * 24, c.y + i * 24, md),
    );
  }

  // Native clipboard peek (the web `paste` event only fires on ⌘V). Returns the
  // clipboard image handle, or null when the clipboard holds no image.
  async function clipboardImage(): Promise<Image | null> {
    try {
      return await readImage();
    } catch {
      return null;
    }
  }

  async function pasteHere() {
    const img = pasteImage;
    const at = pasteMenu;
    pasteMenu = null;
    pasteImage = null;
    if (!img || !at) return;
    const { width, height } = await img.size();
    const rgba = await img.rgba();
    const canvas = document.createElement("canvas");
    canvas.width = width;
    canvas.height = height;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    ctx.putImageData(
      new ImageData(new Uint8ClampedArray(rgba), width, height),
      0,
      0,
    );
    const blob: Blob | null = await new Promise((res) =>
      canvas.toBlob((b) => res(b), "image/png"),
    );
    if (!blob) return;
    const md = await storeAsset(
      new File([blob], "pasted.png", { type: "image/png" }),
    );
    board.add(randomColor(), at.wx, at.wy, md);
  }

  async function onPaste(e: ClipboardEvent) {
    const items = e.clipboardData?.items;
    if (!items) return;
    const files: File[] = [];
    for (const it of items) {
      if (it.kind === "file") {
        const f = it.getAsFile();
        if (f) files.push(f);
      }
    }
    if (!files.length) return; // no file -> let normal text paste run
    e.preventDefault();
    const md = (await Promise.all(files.map(storeAsset))).join("\n");
    const editing = editingId && board.notes.find((n) => n.id === editingId);
    if (editing) {
      appendAssets(editing, md);
    } else {
      const c = viewCenterWorld();
      board.add(randomColor(), c.x, c.y, md);
    }
  }

  function onDragOver(e: DragEvent) {
    if (e.dataTransfer?.types.includes("Files")) {
      e.preventDefault();
      e.dataTransfer.dropEffect = "copy";
    }
  }

  async function onDrop(e: DragEvent) {
    const files = Array.from(e.dataTransfer?.files ?? []);
    if (!files.length) return;
    e.preventDefault();
    const world = toWorld(e.clientX, e.clientY);
    const noteEl = (e.target as HTMLElement).closest(
      "[data-note]",
    ) as HTMLElement | null;
    const refs = await Promise.all(files.map(storeAsset));
    const onto = noteEl && board.notes.find((n) => n.id === noteEl.dataset.note);
    if (onto) {
      appendAssets(onto, refs.join("\n"));
      board.bringToFront(onto);
    } else {
      commitEdit();
      refs.forEach((md, i) =>
        board.add(randomColor(), world.x + i * 24, world.y + i * 24, md),
      );
    }
  }

  async function onContextMenu(e: MouseEvent) {
    e.preventDefault();
    const noteEl = (e.target as HTMLElement).closest(
      "[data-note]",
    ) as HTMLElement | null;

    if (!noteEl) {
      // empty canvas: offer Paste only when the clipboard holds an image
      menu = null;
      pasteMenu = null;
      const cx = e.clientX;
      const cy = e.clientY;
      const world = toWorld(cx, cy);
      const img = await clipboardImage();
      if (!img) return;
      pasteImage = img;
      pasteMenu = {
        x: Math.min(cx, window.innerWidth - 168),
        y: Math.min(cy, window.innerHeight - 54),
        wx: world.x,
        wy: world.y,
      };
      return;
    }

    pasteMenu = null;
    pasteImage = null;
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
    pasteMenu = null;
    const target = e.target as HTMLElement;
    const noteEl = target.closest("[data-note]") as HTMLElement | null;

    if (noteEl) {
      const id = noteEl.dataset.note!;
      if (id === editingId) return;
      // let the audio controls take the pointer instead of dragging the note
      if (target.closest("audio")) {
        const note = board.notes.find((n) => n.id === id);
        if (note) board.bringToFront(note);
        return;
      }
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

  function openAsset(raw: string) {
    const isUrl = /^[a-z][a-z0-9+.-]*:/i.test(raw);
    (isUrl ? openUrl(raw) : openPath(assetPath(raw))).catch(() => {});
  }

  // Shared-element zoom: hand the lightbox the very element the user clicked and
  // hide the on-canvas one, so a single image/video appears to fly out and blur
  // the board, then back. Visibility is restored once the close animation lands.
  function openLightbox(el: HTMLImageElement | HTMLVideoElement, kind: "image" | "video") {
    lightboxOrigin = el.getBoundingClientRect();
    lightboxSourceEl = el;
    lightboxKind = kind;
    el.style.visibility = "hidden";
    lightbox = el instanceof HTMLImageElement ? el.currentSrc || el.src : el.src;
  }

  function closeLightbox() {
    if (lightboxSourceEl) lightboxSourceEl.style.visibility = "";
    lightboxSourceEl = null;
    lightbox = null;
  }

  function onDblClick(e: MouseEvent) {
    const target = e.target as HTMLElement;
    const chip = target.closest(".note-file") as HTMLElement | null;
    if (chip?.dataset.asset) {
      openAsset(chip.dataset.asset);
      return;
    }
    const img = target.closest(".note-img") as HTMLImageElement | null;
    if (img) {
      openLightbox(img, "image");
      return;
    }
    const video = target.closest(".note-media") as HTMLVideoElement | null;
    if (video) {
      openLightbox(video, "video");
      return;
    }
    if (target.closest(".note-audio")) return;
    const noteEl = target.closest("[data-note]") as HTMLElement | null;
    if (!noteEl) return;
    const note = board.notes.find((n) => n.id === noteEl.dataset.note);
    if (!note) return;
    if (isAssetOnly(note.text)) return;
    board.bringToFront(note);
    focusNote(note);
  }

  function onKeyDown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      // the lightbox owns Escape while open (animated close)
      if (lightbox) return;
      if (pasteMenu) {
        pasteMenu = null;
        return;
      }
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

  // ---- dotted grid (canvas, GPU-friendly: zoom is a pattern transform,
  //      not a per-frame gradient re-rasterization, so it never shimmers) ----
  let gridCanvas = $state<HTMLCanvasElement | null>(null);
  let gridCtx: CanvasRenderingContext2D | null = null;
  let dotTile: HTMLCanvasElement | null = null;
  let dotPattern: CanvasPattern | null = null;
  let dpr = 1;

  function buildDotTile() {
    const size = Math.max(1, Math.round(GRID * dpr));
    const c = document.createElement("canvas");
    c.width = size;
    c.height = size;
    const g = c.getContext("2d")!;
    const r = 1.4 * dpr;
    const cx = size / 2;
    const grad = g.createRadialGradient(cx, cx, 0, cx, cx, r + dpr);
    grad.addColorStop(0, "rgba(40, 38, 32, 0.13)");
    grad.addColorStop(r / (r + dpr), "rgba(40, 38, 32, 0.13)");
    grad.addColorStop(1, "rgba(40, 38, 32, 0)");
    g.fillStyle = grad;
    g.beginPath();
    g.arc(cx, cx, r + dpr, 0, Math.PI * 2);
    g.fill();
    dotTile = c;
    dotPattern = null;
  }

  function resizeGrid() {
    if (!gridCanvas) return;
    dpr = window.devicePixelRatio || 1;
    gridCanvas.width = Math.round(window.innerWidth * dpr);
    gridCanvas.height = Math.round(window.innerHeight * dpr);
    gridCanvas.style.width = window.innerWidth + "px";
    gridCanvas.style.height = window.innerHeight + "px";
    gridCtx = gridCanvas.getContext("2d");
    buildDotTile();
    drawGrid();
  }

  function drawGrid() {
    const ctx = gridCtx;
    if (!ctx || !gridCanvas || !dotTile) return;
    if (!dotPattern) dotPattern = ctx.createPattern(dotTile, "repeat");
    if (!dotPattern) return;
    const w = gridCanvas.width;
    const h = gridCanvas.height;
    const s = board.camera.scale;
    ctx.setTransform(1, 0, 0, 1, 0, 0);
    ctx.clearRect(0, 0, w, h);
    ctx.setTransform(s, 0, 0, s, board.camera.x * dpr, board.camera.y * dpr);
    ctx.fillStyle = dotPattern;
    ctx.fillRect(
      (-board.camera.x * dpr) / s,
      (-board.camera.y * dpr) / s,
      w / s,
      h / s,
    );
  }

  $effect(() => {
    board.camera.x;
    board.camera.y;
    board.camera.scale;
    if (gridCanvas) drawGrid();
  });

  onMount(() => {
    resizeGrid();
    window.addEventListener("resize", resizeGrid);
    return () => window.removeEventListener("resize", resizeGrid);
  });

  const worldStyle = $derived(
    `transform:translate(${board.camera.x}px,${board.camera.y}px) scale(${board.camera.scale});`,
  );
  const zoomPct = $derived(Math.round(board.camera.scale * 100));
  const menuAssetOnly = $derived(
    !!menu &&
      isAssetOnly(board.notes.find((n) => n.id === menu!.id)?.text ?? ""),
  );
</script>

<svelte:window
  onpointermove={onPointerMove}
  onpointerup={onPointerUp}
  onpointercancel={endInteraction}
  onkeydown={onKeyDown}
  onpaste={onPaste}
/>

<div class="titlebar" data-ui data-tauri-drag-region></div>

<main
  class="viewport"
  class:panning
  onwheel={onWheel}
  onpointerdown={onPointerDown}
  ondblclick={onDblClick}
  oncontextmenu={onContextMenu}
  ondragover={onDragOver}
  ondrop={onDrop}
>
  <canvas bind:this={gridCanvas} class="grid"></canvas>
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

  <Trash bind:el={trashEl} hot={trashHot} armed={dragId !== null} />

  <div class="dock" data-ui>
    <Palette colors={COLORS} onpick={addNote} />
    <AttachButton onfiles={addAssetNotes} />
  </div>

  <ZoomControls
    zoom={zoomPct}
    onZoomIn={() => zoomBy(1.25)}
    onZoomOut={() => zoomBy(1 / 1.25)}
    onReset={resetView}
  />
</main>

{#if menu}
  <ContextMenu
    x={menu.x}
    y={menu.y}
    colors={COLORS}
    showColors={!menuAssetOnly}
    oncolor={(c) => setColor(menu!.id, c)}
    ondelete={() => deleteFromMenu(menu!.id)}
    onclose={() => (menu = null)}
  />
{/if}

{#if pasteMenu}
  <PasteMenu
    x={pasteMenu.x}
    y={pasteMenu.y}
    onpaste={pasteHere}
    onclose={() => {
      pasteMenu = null;
      pasteImage = null;
    }}
  />
{/if}

{#if lightbox}
  <Lightbox
    src={lightbox}
    kind={lightboxKind}
    origin={lightboxOrigin}
    onclose={closeLightbox}
  />
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
    cursor: default;
  }
  .grid {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    z-index: 0;
    pointer-events: none;
  }
  .viewport.panning {
    cursor: grabbing;
  }
  .world {
    position: absolute;
    top: 0;
    left: 0;
    z-index: 1;
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

  .dock {
    position: fixed;
    left: 50%;
    bottom: 24px;
    z-index: 40;
    transform: translateX(-50%);
    display: flex;
    align-items: center;
    gap: 12px;
  }

  @media (prefers-reduced-motion: reduce) {
    * {
      animation: none !important;
      transition: none !important;
    }
  }
</style>
