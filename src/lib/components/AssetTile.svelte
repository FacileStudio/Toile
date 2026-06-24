<script lang="ts">
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
  // Only show an inline player for formats this webview decodes; everything else
  // — and anything that errors at runtime — falls through to an openable card.
  const playable = $derived(inlinePlayable(raw));
  let failed = $state(false);

  // WKWebView paints nothing for a <video> until it seeks. The #t=0.001 media
  // fragment forces it to decode and paint that first frame as a still poster,
  // with no autoplay; the loadedmetadata seek is a belt-and-suspenders fallback.
  const videoSrc = $derived(`${src}#t=0.001`);

  function poster(e: Event) {
    const v = e.currentTarget as HTMLVideoElement;
    try {
      v.currentTime = 0.001;
    } catch {
      /* seek unsupported — the fragment already handled it */
    }
  }
</script>

{#if kind === "video" && playable && !failed}
  <span class="tile-video">
    <!-- svelte-ignore a11y_media_has_caption -->
    <video
      class="note-media"
      src={videoSrc}
      muted
      playsinline
      preload="metadata"
      draggable="false"
      onloadedmetadata={poster}
      onerror={() => (failed = true)}
    ></video>
    <span class="play" aria-hidden="true">
      <svg viewBox="0 0 24 24" width="22" height="22" fill="currentColor">
        <path d="M8 5v14l11-7z" />
      </svg>
    </span>
  </span>
{:else if kind === "audio" && playable && !failed}
  <span class="tile-audio">
    <span class="tile-head">
      <svg
        class="tile-icon"
        viewBox="0 0 24 24"
        width="18"
        height="18"
        fill="none"
        stroke="currentColor"
        stroke-width="1.7"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <path d="M9 18V5l12-2v13" />
        <circle cx="6" cy="18" r="3" />
        <circle cx="18" cy="16" r="3" />
      </svg>
      <span class="tile-name">{name}</span>
    </span>
    <audio class="note-audio" {src} controls onerror={() => (failed = true)}></audio>
  </span>
{:else}
  <span class="note-file" data-asset={raw} title={name}>
    <svg
      class="tile-icon"
      viewBox="0 0 24 24"
      width="20"
      height="20"
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
  .tile-video {
    position: relative;
    display: block;
    margin: 6px 0;
  }
  .tile-video .note-media {
    display: block;
    width: 100%;
    height: auto;
    border-radius: 8px;
    box-shadow: 0 1px 3px rgba(40, 38, 32, 0.18);
    background: #1c1b18;
    -webkit-user-drag: none;
  }
  .tile-video .play {
    position: absolute;
    inset: 0;
    margin: auto;
    width: 46px;
    height: 46px;
    display: grid;
    place-items: center;
    color: #fff;
    background: rgba(20, 19, 16, 0.5);
    border-radius: 50%;
    pointer-events: none;
    box-shadow: 0 2px 10px rgba(0, 0, 0, 0.35);
    backdrop-filter: blur(2px);
    -webkit-backdrop-filter: blur(2px);
  }
  .tile-video .play svg {
    margin-left: 2px;
  }

  .tile-audio {
    display: flex;
    flex-direction: column;
    gap: 8px;
    width: 100%;
    box-sizing: border-box;
    margin: 4px 0;
    padding: 12px;
    border-radius: 12px;
    background: rgba(255, 255, 255, 0.55);
    box-shadow: inset 0 0 0 1px rgba(40, 38, 32, 0.12);
    color: var(--ink);
  }
  .tile-audio .note-audio {
    width: 100%;
    height: 34px;
  }

  .tile-head {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }
  .tile-icon {
    flex: 0 0 auto;
    opacity: 0.7;
  }

  .note-file {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    box-sizing: border-box;
    margin: 4px 0;
    padding: 10px 13px;
    border-radius: 11px;
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

  .tile-name {
    font-size: 15px;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
