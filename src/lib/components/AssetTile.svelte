<script lang="ts">
  import VideoPlayer from "./VideoPlayer.svelte";
  import AudioPlayer from "./AudioPlayer.svelte";
  import { inlinePlayable, resolveAssetSrc, type AssetKind } from "../assets";

  let {
    kind,
    raw,
    name,
  }: {
    kind: Exclude<AssetKind, "image">;
    raw: string;
    name: string;
  } = $props();

  const src = $derived(resolveAssetSrc(raw));
  // Only hand video/audio to a player for formats this webview decodes; anything
  // else — and anything that errors at runtime — falls through to an open card.
  const playable = $derived(inlinePlayable(raw));
  let failed = $state(false);
</script>

{#if kind === "video" && playable && !failed}
  <VideoPlayer {src} onfail={() => (failed = true)} />
{:else if kind === "audio" && playable && !failed}
  <AudioPlayer {src} {name} onfail={() => (failed = true)} />
{:else}
  <span class="note-file" data-asset={raw} title={name}>
    <svg
      class="tile-icon"
      viewBox="0 0 24 24"
      width="22"
      height="22"
      fill="none"
      stroke="currentColor"
      stroke-width="1.7"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      {#if kind === "video"}
        <rect x="2" y="4" width="20" height="16" rx="2" />
        <path d="m10 9 5 3-5 3z" />
      {:else if kind === "audio"}
        <path d="M9 18V5l12-2v13" />
        <circle cx="6" cy="18" r="3" />
        <circle cx="18" cy="16" r="3" />
      {:else}
        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
        <path d="M14 2v6h6" />
      {/if}
    </svg>
    <span class="tile-name">{name}</span>
  </span>
{/if}

<style>
  .note-file {
    display: flex;
    align-items: center;
    gap: 12px;
    width: 100%;
    box-sizing: border-box;
    margin: 6px 0;
    padding: 16px 18px;
    border-radius: 15px;
    background: rgba(255, 255, 255, 0.55);
    box-shadow: inset 0 0 0 1px rgba(40, 38, 32, 0.12);
    color: var(--ink);
    cursor: pointer;
    transition: background 0.18s ease;
    -webkit-user-drag: none;
  }
  .note-file:hover {
    background: rgba(255, 255, 255, 0.82);
  }
  .tile-icon {
    flex: 0 0 auto;
    width: 28px;
    height: 28px;
    opacity: 0.72;
  }
  .tile-name {
    font-size: 18px;
    font-weight: 600;
    line-height: 1.3;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
