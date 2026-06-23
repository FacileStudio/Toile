<script lang="ts">
  import { scale } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import { isImageOnly, resolveAssetSrc, type Note } from "./board.svelte";

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

  // Split the body into text + image segments. Images are markdown refs
  // (`![](assets/…)`); everything else stays plain text, so the note still
  // reads as a sticky note — it just renders pinned pictures inline.
  type Segment = { img: false; value: string } | { img: true; src: string };

  // A note that is nothing but image(s) sheds the colored card and renders as
  // the bare rounded image at its natural ratio. Text notes keep the card.
  const imageOnly = $derived(isImageOnly(note.text));

  const segments = $derived.by<Segment[]>(() => {
    const out: Segment[] = [];
    const re = /!\[[^\]]*\]\(([^)]+)\)/g;
    let last = 0;
    let m: RegExpExecArray | null;
    while ((m = re.exec(note.text))) {
      if (m.index > last)
        out.push({ img: false, value: note.text.slice(last, m.index) });
      out.push({ img: true, src: resolveAssetSrc(m[1].trim()) });
      last = m.index + m[0].length;
    }
    if (last < note.text.length)
      out.push({ img: false, value: note.text.slice(last) });
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
  class:image-only={imageOnly && !editing}
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
            {#if seg.img}
              <img class="note-img" src={seg.src} alt="" draggable="false" />
            {:else}{seg.value}{/if}
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

  /* image-only note: no card, just the rounded image at its natural ratio */
  .note.image-only {
    height: auto !important;
  }
  .note.image-only .inner {
    height: auto;
    padding: 0;
    background: transparent;
    box-shadow: none;
    overflow: visible;
  }
  .note.image-only:hover .inner {
    box-shadow: none;
  }
  .note.image-only .note-img {
    width: 100%;
    margin: 0;
    border-radius: 14px;
    box-shadow:
      0 1px 2px rgba(40, 38, 32, 0.1),
      0 8px 18px rgba(40, 38, 32, 0.14);
  }
  .note.image-only .note-img + .note-img {
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
