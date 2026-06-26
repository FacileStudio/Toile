<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { resolveAssetSrc } from "../assets";
  import { links } from "../links.svelte";
  import { youtubeId, safeExternal, hostOf } from "../links";

  let { url }: { url: string } = $props();

  // YouTube is detected client-side so the facade paints instantly, with no fetch
  // on the critical path — the backend call only enriches it with a title.
  const vid = $derived(youtubeId(url));
  const meta = $derived(links.get(url) ?? null);
  const host = $derived(hostOf(url));

  let playing = $state(false);

  // Fire the (queued, cached) preview fetch. No-op when cached or disabled.
  $effect(() => {
    void url;
    links.ensure(url);
  });

  const thumb = $derived(vid ? `https://i.ytimg.com/vi/${vid}/hqdefault.jpg` : null);
  const image = $derived(meta?.image ? resolveAssetSrc(meta.image) : null);
  const favicon = $derived(meta?.favicon ? resolveAssetSrc(meta.favicon) : null);

  const title = $derived(meta?.title || (vid ? "" : host));
  const loading = $derived(!vid && !meta && links.enabled);

  function open() {
    if (safeExternal(url)) openUrl(url).catch(() => {});
  }

  function onClick(e: MouseEvent) {
    e.stopPropagation();
    if (vid) {
      playing = true;
      return;
    }
    open();
  }
</script>

{#if vid}
  <!-- facade: thumbnail + play button until clicked, then the real player.
       not draggable once playing so the iframe owns its own pointer events. -->
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div
    class="link yt"
    class:playing
    data-link={url}
    data-interactive={playing ? "" : undefined}
    role="button"
    tabindex="-1"
    onclick={playing ? undefined : onClick}
  >
    {#if playing}
      <iframe
        class="yt-frame"
        src={`https://www.youtube-nocookie.com/embed/${vid}?autoplay=1&rel=0&playsinline=1`}
        title={title || "YouTube video"}
        allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share; fullscreen"
        referrerpolicy="strict-origin-when-cross-origin"
        allowfullscreen
      ></iframe>
    {:else}
      <img class="yt-thumb" src={thumb} alt={title || "video thumbnail"} draggable="false" />
      <span class="yt-play" aria-hidden="true">
        <svg viewBox="0 0 68 48" width="58" height="40">
          <path
            class="yt-play-bg"
            d="M66.5 7.7a8 8 0 0 0-5.6-5.7C56 0.7 34 0.7 34 0.7s-22 0-26.9 1.3A8 8 0 0 0 1.5 7.7C0.2 12.6 0.2 24 0.2 24s0 11.4 1.3 16.3a8 8 0 0 0 5.6 5.7C12 47.3 34 47.3 34 47.3s22 0 26.9-1.3a8 8 0 0 0 5.6-5.7C67.8 35.4 67.8 24 67.8 24s0-11.4-1.3-16.3z"
          />
          <path d="M27 34V14l18 10z" fill="#fff" />
        </svg>
      </span>
      {#if title}<span class="yt-title">{title}</span>{/if}
    {/if}
  </div>
{:else if meta?.kind === "card"}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="link card" data-link={url} role="button" tabindex="-1" onclick={onClick}
       style={meta.accent ? `--accent:${meta.accent}` : ""}>
    {#if image}
      <img class="card-img" src={image} alt="" draggable="false" />
    {/if}
    <div class="card-body">
      <span class="card-site">
        {#if favicon}<img class="card-fav" src={favicon} alt="" draggable="false" />{/if}
        <span class="card-host">{meta.siteName || host}</span>
      </span>
      {#if title}<span class="card-title">{title}</span>{/if}
      {#if meta.description}<span class="card-desc">{meta.description}</span>{/if}
    </div>
  </div>
{:else}
  <!-- loading skeleton, or the graceful fallback chip when previews are off /
       the fetch failed: still a tidy, clickable link. -->
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="link chip" class:loading data-link={url} role="button" tabindex="-1" onclick={onClick}>
    {#if favicon}<img class="card-fav" src={favicon} alt="" draggable="false" />{/if}
    <span class="chip-text">{loading ? "Loading preview…" : host}</span>
  </div>
{/if}

<style>
  .link {
    display: block;
    width: 100%;
    box-sizing: border-box;
    margin: 6px 0;
    cursor: pointer;
    -webkit-user-drag: none;
    user-select: none;
  }

  /* ---- YouTube facade ---- */
  .yt {
    position: relative;
    aspect-ratio: 16 / 9;
    border-radius: 14px;
    overflow: hidden;
    background: #000;
    box-shadow: inset 0 0 0 1px rgba(40, 38, 32, 0.12);
  }
  .yt-thumb {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }
  .yt-frame {
    width: 100%;
    height: 100%;
    border: 0;
    display: block;
  }
  .yt-play {
    position: absolute;
    inset: 0;
    display: grid;
    place-items: center;
    transition: transform 0.18s var(--ease-soft, ease);
  }
  .yt-play-bg {
    fill: #212121;
    opacity: 0.82;
    transition: fill 0.18s ease, opacity 0.18s ease;
  }
  .yt:hover .yt-play {
    transform: scale(1.06);
  }
  .yt:hover .yt-play-bg {
    fill: #f00;
    opacity: 1;
  }
  .yt-title {
    position: absolute;
    left: 0;
    right: 0;
    bottom: 0;
    padding: 26px 12px 9px;
    font-size: 14px;
    font-weight: 600;
    line-height: 1.3;
    color: #fff;
    background: linear-gradient(transparent, rgba(0, 0, 0, 0.72));
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  /* ---- bookmark card ---- */
  .card {
    display: flex;
    gap: 0;
    border-radius: 14px;
    overflow: hidden;
    background: rgba(255, 255, 255, 0.62);
    box-shadow: inset 0 0 0 1px rgba(40, 38, 32, 0.12);
    border-left: 3px solid var(--accent, rgba(40, 38, 32, 0.18));
    transition: background 0.18s ease;
  }
  .card:hover {
    background: rgba(255, 255, 255, 0.85);
  }
  .card-img {
    flex: 0 0 88px;
    width: 88px;
    align-self: stretch;
    object-fit: cover;
    background: rgba(40, 38, 32, 0.06);
  }
  .card-body {
    display: flex;
    flex-direction: column;
    gap: 3px;
    padding: 11px 13px;
    min-width: 0;
  }
  .card-site {
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
  }
  .card-fav {
    width: 15px;
    height: 15px;
    border-radius: 3px;
    flex: 0 0 auto;
    object-fit: contain;
  }
  .card-host {
    font-size: 12px;
    font-weight: 600;
    color: var(--ink-soft, rgba(67, 65, 59, 0.7));
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .card-title {
    font-size: 15px;
    font-weight: 700;
    line-height: 1.3;
    color: var(--ink, #1f1d19);
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
  .card-desc {
    font-size: 13px;
    font-weight: 500;
    line-height: 1.35;
    color: var(--ink-soft, rgba(67, 65, 59, 0.66));
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  /* ---- compact chip (loading / fallback) ---- */
  .chip {
    display: flex;
    align-items: center;
    gap: 9px;
    padding: 11px 14px;
    border-radius: 13px;
    background: rgba(255, 255, 255, 0.55);
    box-shadow: inset 0 0 0 1px rgba(40, 38, 32, 0.12);
    transition: background 0.18s ease;
  }
  .chip:hover {
    background: rgba(255, 255, 255, 0.82);
  }
  .chip-text {
    font-size: 15px;
    font-weight: 600;
    color: var(--ink, #1f1d19);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .chip.loading {
    opacity: 0.7;
  }
  .chip.loading .chip-text {
    color: var(--ink-soft, rgba(67, 65, 59, 0.6));
  }
</style>
