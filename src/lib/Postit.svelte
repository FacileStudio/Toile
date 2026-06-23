<script lang="ts">
  import { scale } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import type { Note } from "./board.svelte";

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
        {note.text || "Write something…"}
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
