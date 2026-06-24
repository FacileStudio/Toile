<script lang="ts">
  import { scale } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import AssetTile from "./AssetTile.svelte";
  import {
    assetKind,
    isAssetOnly,
    resolveAssetSrc,
    type AssetKind,
  } from "../assets";
  import type { Note } from "../board.svelte";

  let {
    note,
    editing = false,
    dragging = false,
    doomed = false,
  }: {
    note: Note;
    editing?: boolean;
    dragging?: boolean;
    doomed?: boolean;
  } = $props();

  let textarea = $state<HTMLTextAreaElement | null>(null);

  // Split the body into text + asset segments. `![alt](path)` embeds carry media
  // (image inline, video/audio via AssetTile); `[name](path)` links render as a
  // click-to-open file tile. Everything else stays plain text, so a note still
  // reads as a sticky — it just pins whatever you dropped on it.
  type Segment =
    | { kind: "text"; value: string }
    | { kind: "image"; src: string; alt: string }
    | { kind: Exclude<AssetKind, "image">; raw: string; name: string };

  // A note that is nothing but asset refs sheds the colored card and renders as
  // its own asset component. Notes with prose keep the sticky card.
  const assetOnly = $derived(isAssetOnly(note.text));

  const fileName = (label: string, raw: string) =>
    label.trim() || (raw.split("/").pop() ?? raw);

  const segments = $derived.by<Segment[]>(() => {
    const out: Segment[] = [];
    const re = /(!?)\[([^\]]*)\]\(([^)]+)\)/g;
    let last = 0;
    let m: RegExpExecArray | null;
    while ((m = re.exec(note.text))) {
      if (m.index > last)
        out.push({ kind: "text", value: note.text.slice(last, m.index) });
      const raw = m[3].trim();
      const name = fileName(m[2], raw);
      const kind = m[1] === "!" ? assetKind(raw) : "file";
      if (kind === "image")
        out.push({ kind, src: resolveAssetSrc(raw), alt: name });
      else out.push({ kind, raw, name });
      last = m.index + m[0].length;
    }
    if (last < note.text.length)
      out.push({ kind: "text", value: note.text.slice(last) });
    return out;
  });

  $effect(() => {
    if (editing && textarea) {
      textarea.focus();
      const len = textarea.value.length;
      textarea.setSelectionRange(len, len);
    }
  });
</script>

<div
  class="note"
  class:editing
  class:dragging
  class:doomed
  class:asset-only={assetOnly && !editing}
  data-note={note.id}
  style="left:{note.x}px; top:{note.y}px; width:{note.w}px; height:{note.h}px; z-index:{note.z}; --bg:{note.color}"
  in:scale={{ duration: 280, start: 0.82, opacity: 0, easing: cubicOut }}
  out:scale={{ duration: 200, start: 0.78, opacity: 0, easing: cubicOut }}
>
  <div class="inner">
    {#if editing}
      <textarea
        bind:this={textarea}
        bind:value={note.text}
        placeholder="Write something…"
        spellcheck="false"
      ></textarea>
    {:else}
      <div class="text" class:empty={!note.text}>
        {#if note.text}
          {#each segments as seg}
            {#if seg.kind === "image"}
              <img class="note-img" src={seg.src} alt={seg.alt} draggable="false" />
            {:else if seg.kind === "text"}{seg.value}{:else}
              <AssetTile kind={seg.kind} raw={seg.raw} name={seg.name} />
            {/if}
          {/each}
        {:else}
          Write something…
        {/if}
      </div>
    {/if}
  </div>
</div>

<style>
  .note {
    position: absolute;
    border-radius: 14px;
    will-change: transform;
  }

  .inner {
    width: 100%;
    height: 100%;
    border-radius: inherit;
    background: var(--bg);
    padding: 18px;
    box-shadow:
      0 1px 2px rgba(40, 38, 32, 0.08),
      0 8px 18px rgba(40, 38, 32, 0.1);
    transition:
      transform 0.28s var(--ease-soft),
      box-shadow 0.28s var(--ease-soft);
    cursor: grab;
    overflow: hidden;
  }

  .note:hover .inner {
    transform: translateY(-3px);
    box-shadow:
      0 2px 4px rgba(40, 38, 32, 0.1),
      0 14px 30px rgba(40, 38, 32, 0.16);
  }

  .note.dragging .inner {
    cursor: grabbing;
    transform: translateY(-4px) scale(1.02) rotate(-1.2deg);
    box-shadow:
      0 4px 8px rgba(40, 38, 32, 0.14),
      0 26px 50px rgba(40, 38, 32, 0.24);
  }

  .note.editing .inner {
    transform: scale(1.015);
    cursor: text;
    box-shadow:
      0 3px 6px rgba(40, 38, 32, 0.12),
      0 22px 46px rgba(40, 38, 32, 0.2);
  }

  .note.doomed .inner {
    transform: scale(0.82) rotate(-3deg);
    opacity: 0.55;
  }

  .text,
  textarea {
    width: 100%;
    height: 100%;
    font-family: inherit;
    font-size: 19px;
    line-height: 1.45;
    font-weight: 500;
    color: var(--ink);
    white-space: pre-wrap;
    word-break: break-word;
    overflow-wrap: break-word;
  }

  .text.empty {
    color: rgba(67, 65, 59, 0.38);
  }

  .note-img {
    display: block;
    max-width: 100%;
    height: auto;
    margin: 6px 0;
    border-radius: 8px;
    box-shadow: 0 1px 3px rgba(40, 38, 32, 0.18);
    user-select: none;
    -webkit-user-drag: none;
  }

  /* asset-only note: no card, the asset renders as its own component */
  .note.asset-only {
    height: auto !important;
  }
  .note.asset-only .inner {
    height: auto;
    padding: 0;
    background: transparent;
    box-shadow: none;
    overflow: visible;
  }
  .note.asset-only:hover .inner {
    box-shadow: none;
  }
  .note.asset-only .note-img {
    width: 100%;
    margin: 0;
    border-radius: 14px;
    box-shadow:
      0 1px 2px rgba(40, 38, 32, 0.1),
      0 8px 18px rgba(40, 38, 32, 0.14);
  }
  .note.asset-only .note-img + .note-img {
    margin-top: 6px;
  }

  textarea {
    border: none;
    outline: none;
    resize: none;
    background: transparent;
    user-select: text;
    -webkit-user-select: text;
  }

  textarea::placeholder {
    color: rgba(67, 65, 59, 0.38);
  }
</style>
