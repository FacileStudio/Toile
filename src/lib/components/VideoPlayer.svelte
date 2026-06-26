<script lang="ts">
  let {
    src,
    onfail,
    focused = false,
  }: { src: string; onfail?: () => void; focused?: boolean } = $props();

  const posterSrc = $derived(`${src}#t=0.001`);

  let video = $state<HTMLVideoElement | null>(null);
  let playing = $state(false);
  let cur = $state(0);
  let dur = $state(0);

  const pct = $derived(dur > 0 ? Math.min(100, (cur / dur) * 100) : 0);

  function toggle(e: Event) {
    e.stopPropagation();
    const v = video;
    if (!v) return;
    if (v.paused) v.play().catch(() => onfail?.());
    else v.pause();
  }

  function frame(e: Event) {
    const v = e.currentTarget as HTMLVideoElement;
    try {
      v.currentTime = 0.001;
    } catch {
    }
  }

  function seek(e: PointerEvent) {
    e.stopPropagation();
    const v = video;
    if (!v || !dur) return;
    const r = (e.currentTarget as HTMLElement).getBoundingClientRect();
    v.currentTime = Math.max(0, Math.min(1, (e.clientX - r.left) / r.width)) * dur;
  }
</script>

<div class="vplayer" class:playing class:focused>
  <!-- svelte-ignore a11y_media_has_caption -->
  <video
    bind:this={video}
    class="note-media"
    src={posterSrc}
    playsinline
    preload="metadata"
    draggable="false"
    onloadedmetadata={frame}
    onloadeddata={() => (dur = video?.duration ?? 0)}
    onplay={() => (playing = true)}
    onpause={() => (playing = false)}
    onended={() => (playing = false)}
    ontimeupdate={() => (cur = video?.currentTime ?? 0)}
    onerror={() => onfail?.()}
  ></video>

  <button
    class="vid-play vid-ctl"
    onclick={toggle}
    onpointerdown={(e) => e.stopPropagation()}
    aria-label={playing ? "Pause" : "Play"}
  >
    {#if playing}
      <svg viewBox="0 0 24 24" width="24" height="24" fill="currentColor">
        <rect x="6" y="5" width="4" height="14" rx="1" />
        <rect x="14" y="5" width="4" height="14" rx="1" />
      </svg>
    {:else}
      <svg viewBox="0 0 24 24" width="24" height="24" fill="currentColor">
        <path d="M8 5v14l11-7z" />
      </svg>
    {/if}
  </button>

  <div
    class="vbar vid-ctl"
    onpointerdown={seek}
    role="slider"
    aria-label="Seek"
    aria-valuenow={Math.round(pct)}
    aria-valuemin="0"
    aria-valuemax="100"
    tabindex="-1"
  >
    <span class="vfill" style="width:{pct}%"></span>
  </div>
</div>

<style>
  .vplayer {
    position: relative;
    display: block;
    margin: 6px 0;
    border-radius: 12px;
    overflow: hidden;
    font-size: 0;
    line-height: 0;
  }
  .note-media {
    display: block;
    width: 100%;
    height: auto;
    object-fit: cover;
    -webkit-user-drag: none;
  }

  .vid-play {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: 52px;
    height: 52px;
    display: grid;
    place-items: center;
    border: none;
    border-radius: 50%;
    background: rgba(20, 19, 16, 0.52);
    color: #fff;
    cursor: pointer;
    backdrop-filter: blur(3px);
    -webkit-backdrop-filter: blur(3px);
    box-shadow: 0 2px 12px rgba(0, 0, 0, 0.35);
    transition:
      opacity 0.22s ease,
      transform 0.16s var(--ease-soft);
  }
  .vid-play svg {
    margin-left: 1px;
  }
  .vid-play:hover {
    transform: translate(-50%, -50%) scale(1.08);
  }
  .vid-play:active {
    transform: translate(-50%, -50%) scale(0.94);
  }
  .vplayer.playing .vid-play {
    opacity: 0;
  }
  .vplayer.playing:hover .vid-play {
    opacity: 1;
  }

  .vbar {
    position: absolute;
    left: 8px;
    right: 8px;
    bottom: 8px;
    height: 12px;
    display: flex;
    align-items: center;
    cursor: pointer;
    opacity: 0;
    transition: opacity 0.22s ease;
    touch-action: none;
  }
  .vbar::before {
    content: "";
    position: absolute;
    left: 0;
    right: 0;
    height: 3px;
    border-radius: 99px;
    background: rgba(255, 255, 255, 0.3);
  }
  .vfill {
    position: relative;
    height: 3px;
    border-radius: 99px;
    background: #fff;
    box-shadow: 0 0 6px rgba(255, 255, 255, 0.5);
  }
  .vplayer.playing .vbar,
  .vplayer:hover .vbar,
  .vplayer.focused .vbar {
    opacity: 1;
  }
  .vplayer.focused .vid-play {
    opacity: 1;
  }
</style>
