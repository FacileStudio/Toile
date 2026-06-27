<script lang="ts">
  import { scale } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import AssetTile from "./AssetTile.svelte";
  import LinkCard from "./LinkCard.svelte";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import {
    assetKind,
    isCardOnly,
    resolveAssetSrc,
    type AssetKind,
  } from "../assets";
  import { isBareUrl, safeExternal } from "../links";
  import { renderMarkdown, toggleTask } from "../markdown";
  import { FONT_STACKS } from "../fonts";
  import { TEXT_COLOR, type Note } from "../board.svelte";

  let {
    note,
    editing = false,
    selected = false,
    dragging = false,
    doomed = false,
    dimmed = false,
    focused = false,
  }: {
    note: Note;
    editing?: boolean;
    selected?: boolean;
    dragging?: boolean;
    doomed?: boolean;
    dimmed?: boolean;
    focused?: boolean;
  } = $props();

  let textarea = $state<HTMLTextAreaElement | null>(null);
  let textEl = $state<HTMLDivElement | null>(null);

  function onToggle(e: Event) {
    const box = e.target as HTMLElement;
    if (!(box instanceof HTMLInputElement) || box.type !== "checkbox") return;
    const boxes = [...(textEl?.querySelectorAll("input.md-task") ?? [])];
    const i = boxes.indexOf(box);
    if (i >= 0) note.text = toggleTask(note.text, i);
  }

  function onLinkClick(e: MouseEvent) {
    const a = (e.target as HTMLElement).closest("a.md-link") as HTMLAnchorElement | null;
    if (!a) return;
    e.preventDefault();
    e.stopPropagation();
    const href = a.getAttribute("href") ?? "";
    if (safeExternal(href)) openUrl(href).catch(() => {});
  }

  type Segment =
    | { kind: "text"; value: string }
    | { kind: "link"; url: string }
    | { kind: "image"; src: string; alt: string }
    | { kind: Exclude<AssetKind, "image">; raw: string; name: string };

  const cardOnly = $derived(isCardOnly(note.text));
  const textNote = $derived(note.color === TEXT_COLOR);
  const fontFamily = $derived(FONT_STACKS[note.font ?? "sans"] ?? FONT_STACKS.sans);

  const fileName = (label: string, raw: string) =>
    label.trim() || (raw.split("/").pop() ?? raw);

  function pushText(out: Segment[], chunk: string) {
    let buf: string[] = [];
    const flush = () => {
      if (buf.join("\n").trim()) out.push({ kind: "text", value: buf.join("\n") });
      buf = [];
    };
    for (const line of chunk.split(/\r?\n/)) {
      if (isBareUrl(line)) {
        flush();
        out.push({ kind: "link", url: line.trim() });
      } else {
        buf.push(line);
      }
    }
    flush();
  }

  const segments = $derived.by<Segment[]>(() => {
    const out: Segment[] = [];
    const re = /(!?)\[([^\]]*)\]\(([^)]+)\)/g;
    let last = 0;
    let m: RegExpExecArray | null;
    while ((m = re.exec(note.text))) {
      if (m.index > last) pushText(out, note.text.slice(last, m.index));
      const raw = m[3].trim();
      const name = fileName(m[2], raw);
      const kind = m[1] === "!" ? assetKind(raw) : "file";
      if (kind === "image")
        out.push({ kind, src: resolveAssetSrc(raw), alt: name });
      else out.push({ kind, raw, name });
      last = m.index + m[0].length;
    }
    if (last < note.text.length) pushText(out, note.text.slice(last));
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
  class:selected={selected && !editing}
  class:dragging
  class:doomed
  class:dimmed
  class:focused
  class:asset-only={cardOnly && !editing}
  class:text-note={textNote}
  data-note={note.id}
  style="left:{note.x}px; top:{note.y}px; width:{note.w}px; height:{note.h}px; z-index:{note.z}; --bg:{note.color}; --family:{fontFamily}; --maxw:{note.w}px"
  in:scale={{ duration: 280, start: 0.82, opacity: 0, easing: cubicOut }}
  out:scale={{ duration: 200, start: 0.78, opacity: 0, easing: cubicOut }}
>
  <div class="inner">
    {#if editing}
      {#if textNote}
        <div class="grow">
          <div class="ghost" aria-hidden="true">{(note.text || "Write something…") + " "}</div>
          <textarea
            bind:this={textarea}
            bind:value={note.text}
            placeholder="Write something…"
            spellcheck="false"
          ></textarea>
        </div>
      {:else}
        <textarea
          bind:this={textarea}
          bind:value={note.text}
          placeholder="Write something…"
          spellcheck="false"
        ></textarea>
      {/if}
    {:else}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="text"
        class:empty={!note.text}
        bind:this={textEl}
        onchange={onToggle}
        onclick={onLinkClick}
      >
        {#if note.text}
          {#each segments as seg}
            {#if seg.kind === "image"}
              <img class="note-img" src={seg.src} alt={seg.alt} draggable="false" />
            {:else if seg.kind === "link"}
              <LinkCard url={seg.url} width={cardOnly ? note.w : note.w - 36} />
            {:else if seg.kind === "text"}{@html renderMarkdown(seg.value)}{:else}
              <AssetTile kind={seg.kind} raw={seg.raw} name={seg.name} {focused} />
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
      box-shadow 0.28s var(--ease-soft),
      filter 0.34s var(--ease-soft),
      opacity 0.34s var(--ease-soft);
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

  .note.selected .inner {
    outline: 2px solid var(--ink-soft);
    outline-offset: 2px;
  }
  .note.asset-only.selected .inner {
    outline-offset: 0;
  }
  .note.text-note.selected .inner {
    outline: none;
  }
  .note.text-note.selected .text {
    outline: 2px solid var(--ink-soft);
    outline-offset: 5px;
    border-radius: 7px;
  }

  .note.doomed .inner {
    transform: scale(0.82) rotate(-3deg);
    opacity: 0.55;
  }

  .note.dimmed {
    pointer-events: none;
  }
  .note.dimmed .inner {
    filter: blur(5px);
  }
  .note.focused .inner {
    filter: drop-shadow(0 22px 48px rgba(40, 38, 32, 0.34));
  }

  .note.text-note {
    height: auto !important;
  }
  .note.text-note .inner {
    height: auto;
    background: transparent;
    box-shadow: none;
  }
  .note.text-note .text {
    width: max-content;
    max-width: 100%;
    height: auto;
  }
  .note.text-note:hover .inner {
    transform: none;
    box-shadow: none;
  }
  .note.text-note.editing {
    height: auto !important;
  }
  .note.text-note.editing .inner {
    height: auto;
    padding: 0;
    overflow: visible;
    background: transparent;
    box-shadow: none;
  }

  .grow {
    position: relative;
    display: inline-block;
    max-width: 100%;
    padding: 14px 18px;
    border-radius: 14px;
    background: rgba(255, 255, 255, 0.45);
    box-shadow: 0 10px 28px rgba(40, 38, 32, 0.14);
    transform: scale(1.015);
    transform-origin: top left;
  }
  .grow .ghost,
  .grow > textarea {
    margin: 0;
    font-family: var(--family, inherit);
    font-size: 19px;
    line-height: 1.45;
    font-weight: 500;
    color: var(--ink);
    white-space: pre-wrap;
    word-break: normal;
    overflow-wrap: break-word;
  }
  .grow .ghost {
    padding: 0;
    min-width: 1ch;
    visibility: hidden;
  }
  .grow > textarea {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    padding: 14px 18px;
    box-sizing: border-box;
    overflow: hidden;
    resize: none;
  }


  .text,
  textarea {
    width: 100%;
    height: 100%;
    font-family: var(--family, inherit);
    font-size: 19px;
    line-height: 1.45;
    font-weight: 500;
    color: var(--ink);
    word-break: break-word;
    overflow-wrap: break-word;
  }

  textarea {
    white-space: pre-wrap;
  }

  .text.empty {
    color: rgba(67, 65, 59, 0.38);
  }

  .text :global(p),
  .text :global(ul),
  .text :global(ol),
  .text :global(pre),
  .text :global(blockquote) {
    margin: 0 0 0.5em;
  }
  .text :global(*:last-child) {
    margin-bottom: 0;
  }
  .text :global(h1),
  .text :global(h2),
  .text :global(h3),
  .text :global(h4) {
    margin: 0.3em 0 0.35em;
    line-height: 1.2;
    font-weight: 700;
  }
  .text :global(h1) {
    font-size: 1.4em;
  }
  .text :global(h2) {
    font-size: 1.2em;
  }
  .text :global(h3),
  .text :global(h4) {
    font-size: 1.05em;
  }
  .text :global(strong) {
    font-weight: 700;
  }
  .text :global(em) {
    font-style: italic;
  }
  .text :global(del) {
    opacity: 0.6;
  }
  .text :global(ul) {
    list-style: none;
    padding-left: 0;
  }
  .text :global(ol) {
    padding-left: 1.4em;
  }
  .text :global(li) {
    margin: 0.12em 0;
  }
  .text :global(ul > li) {
    position: relative;
    padding-left: 1.5em;
  }
  .text :global(code) {
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 0.86em;
    background: rgba(40, 38, 32, 0.08);
    padding: 0.08em 0.34em;
    border-radius: 5px;
  }
  .text :global(pre) {
    background: rgba(40, 38, 32, 0.08);
    padding: 10px 12px;
    border-radius: 10px;
    overflow-x: auto;
  }
  .text :global(pre code) {
    background: none;
    padding: 0;
  }
  .text :global(blockquote) {
    padding-left: 0.8em;
    border-left: 3px solid rgba(40, 38, 32, 0.22);
    color: var(--ink-soft);
  }
  .text :global(a) {
    color: inherit;
    text-decoration: underline;
    text-underline-offset: 2px;
    cursor: pointer;
  }
  .text :global(a:hover) {
    text-decoration-thickness: 2px;
  }
  .text :global(hr) {
    border: none;
    border-top: 1px solid rgba(40, 38, 32, 0.18);
    margin: 0.6em 0;
  }

  .text :global(ul > li:not(:has(.md-task)))::before {
    content: "";
    position: absolute;
    left: 0.46em;
    top: 0.55em;
    width: 0.34em;
    height: 0.34em;
    border-radius: 50%;
    background: #1f1d19;
  }
  .text :global(.md-task) {
    appearance: none;
    -webkit-appearance: none;
    position: absolute;
    font-size: inherit;
    left: 0.18em;
    top: 0.275em;
    width: 0.9em;
    height: 0.9em;
    margin: 0;
    border: 1.6px solid #1f1d19;
    border-radius: 50%;
    background: none;
    cursor: pointer;
  }
  .text :global(.md-task:checked) {
    border-color: var(--ink-soft);
  }
  .text :global(.md-task:checked)::after {
    content: "";
    position: absolute;
    left: 50%;
    top: 47%;
    width: 0.24em;
    height: 0.46em;
    border: solid var(--ink-soft);
    border-width: 0 1.7px 1.7px 0;
    transform: translate(-50%, -58%) rotate(45deg);
  }
  .text :global(li:has(.md-task:checked)) {
    color: var(--ink-soft);
    text-decoration: line-through;
  }
  .text :global(li:has(.md-task:checked) *) {
    text-decoration: line-through;
  }

  .note-img {
    display: block;
    max-width: 100%;
    height: auto;
    margin: 6px 0;
    border-radius: 8px;
    user-select: none;
    -webkit-user-drag: none;
  }

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
  .note.asset-only:hover .inner,
  .note.asset-only.selected:hover .inner {
    box-shadow: none;
    transform: none;
  }
  .note.asset-only .note-img {
    width: 100%;
    margin: 0;
    border-radius: 14px;
  }
  .note.asset-only .note-img + .note-img {
    margin-top: 6px;
  }
  .note.asset-only :global(.vplayer),
  .note.asset-only :global(.note-audio),
  .note.asset-only :global(.link) {
    margin: 0;
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
