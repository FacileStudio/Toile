<script lang="ts">
  import { onMount } from "svelte";
  import AudioPlayer from "./AudioPlayer.svelte";

  let {
    src,
    kind = "image",
    name = "",
    origin = null,
    onclose,
  }: {
    src: string;
    kind?: "image" | "video" | "audio";
    name?: string;
    origin?: DOMRect | null;
    onclose: () => void;
  } = $props();

  const DUR = 380;
  const EASE = "cubic-bezier(0.22, 1, 0.36, 1)";
  // lighter than before, and animated from clear -> on over the same DUR as the
  // hero so the blur ramps in progressively and lands exactly when it does.
  const BLUR_ON = "blur(14px) saturate(140%)";
  const BLUR_OFF = "blur(0px) saturate(100%)";
  const DIM_ON = "rgba(40, 38, 32, 0.26)";
  const DIM_OFF = "rgba(40, 38, 32, 0)";
  // the drop shadow fades with the same curve as the FLIP so it never pops when
  // the hero lands and hands off to the (lightly-shadowed) source card.
  const HAIR = "0 0 0 1px rgba(255, 255, 255, 0.12)";
  const SHADOW_ON = `0 24px 60px rgba(0, 0, 0, 0.36), ${HAIR}`;
  const SHADOW_OFF = `0 24px 60px rgba(0, 0, 0, 0), ${HAIR}`;

  let rootEl = $state<HTMLDivElement | null>(null);
  let backdropEl = $state<HTMLDivElement | null>(null);
  let heroEl = $state<HTMLImageElement | HTMLVideoElement | null>(null);
  let audioWrapEl = $state<HTMLDivElement | null>(null);
  let chromeEl = $state<HTMLDivElement | null>(null);
  let closing = false;

  let playing = $state(false);
  let cur = $state(0);
  let dur = $state(0);
  const pct = $derived(dur > 0 ? Math.min(100, (cur / dur) * 100) : 0);

  // FLIP: maps the laid-out fullscreen hero back onto its on-canvas origin rect.
  function originTransform(): string | null {
    if (!origin || !heroEl) return null;
    const f = heroEl.getBoundingClientRect();
    if (!f.width || !f.height) return null;
    const dx = origin.left + origin.width / 2 - (f.left + f.width / 2);
    const dy = origin.top + origin.height / 2 - (f.top + f.height / 2);
    const sx = origin.width / f.width;
    const sy = origin.height / f.height;
    return `translate(${dx}px, ${dy}px) scale(${sx}, ${sy})`;
  }

  function blurFrames(from: { b: string; d: string }, to: { b: string; d: string }): Keyframe[] {
    return [
      { backdropFilter: from.b, webkitBackdropFilter: from.b, background: from.d },
      { backdropFilter: to.b, webkitBackdropFilter: to.b, background: to.d },
    ] as Keyframe[];
  }

  async function ready() {
    if (heroEl instanceof HTMLImageElement) {
      try {
        await heroEl.decode();
      } catch {
        /* not decodable — fall back to a plain transform */
      }
    } else if (heroEl instanceof HTMLVideoElement && heroEl.readyState < 1) {
      await new Promise<void>((r) =>
        heroEl!.addEventListener("loadedmetadata", () => r(), { once: true }),
      );
    }
  }

  // Only the backdrop + chrome fade and the blur ramps in; the hero purely
  // transforms (FLIP) so it reads as one element flying out, never a cross-fade.
  function animateOpen() {
    if (!rootEl) return;
    rootEl.style.opacity = "1";
    backdropEl?.animate(blurFrames({ b: BLUR_OFF, d: DIM_OFF }, { b: BLUR_ON, d: DIM_ON }), {
      duration: DUR,
      easing: EASE,
    });
    chromeEl?.animate([{ opacity: 0 }, { opacity: 1 }], { duration: DUR, easing: EASE });
    if (kind === "audio") {
      audioWrapEl?.animate(
        [{ opacity: 0, transform: "scale(0.92)" }, { opacity: 1, transform: "none" }],
        { duration: DUR, easing: EASE },
      );
      return;
    }
    const from = originTransform();
    if (from && heroEl) {
      heroEl.animate(
        [
          { transform: from, boxShadow: SHADOW_OFF },
          { transform: "none", boxShadow: SHADOW_ON },
        ],
        { duration: DUR, easing: EASE },
      );
    }
  }

  function close() {
    if (closing) return;
    closing = true;
    if (!rootEl) return onclose();
    if (heroEl instanceof HTMLVideoElement) heroEl.pause();
    const opts = { duration: DUR, easing: EASE, fill: "forwards" as const };
    backdropEl?.animate(blurFrames({ b: BLUR_ON, d: DIM_ON }, { b: BLUR_OFF, d: DIM_OFF }), opts);
    chromeEl?.animate([{ opacity: 1 }, { opacity: 0 }], opts);

    let anim: Animation | undefined;
    if (kind === "audio") {
      anim = audioWrapEl?.animate(
        [{ opacity: 1, transform: "none" }, { opacity: 0, transform: "scale(0.92)" }],
        opts,
      );
    } else {
      const to = originTransform();
      anim =
        to && heroEl
          ? heroEl.animate(
              [
                { transform: "none", boxShadow: SHADOW_ON },
                { transform: to, boxShadow: SHADOW_OFF },
              ],
              opts,
            )
          : backdropEl?.animate([{ opacity: 1 }, { opacity: 0.999 }], opts);
    }
    if (anim) anim.onfinish = () => onclose();
    else onclose();
  }

  function toggle(e: Event) {
    e.stopPropagation();
    const v = heroEl;
    if (!(v instanceof HTMLVideoElement)) return;
    if (v.paused) v.play().catch(() => {});
    else v.pause();
  }

  function seek(e: PointerEvent) {
    e.stopPropagation();
    const v = heroEl;
    if (!(v instanceof HTMLVideoElement) || !dur) return;
    const r = (e.currentTarget as HTMLElement).getBoundingClientRect();
    v.currentTime = Math.max(0, Math.min(1, (e.clientX - r.left) / r.width)) * dur;
  }

  function fmt(t: number): string {
    if (!isFinite(t) || t < 0) return "0:00";
    const m = Math.floor(t / 60);
    const s = Math.floor(t % 60);
    return `${m}:${s.toString().padStart(2, "0")}`;
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.stopPropagation();
      close();
    } else if (e.key === " " && kind === "video") {
      e.preventDefault();
      toggle(e);
    }
  }

  onMount(() => {
    (async () => {
      await ready();
      animateOpen();
    })();
    window.addEventListener("keydown", onKey, true);
    return () => window.removeEventListener("keydown", onKey, true);
  });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<div class="lightbox" bind:this={rootEl} data-ui onclick={close}>
  <div class="backdrop" bind:this={backdropEl}></div>

  <div class="stage">
    {#if kind === "video"}
      <!-- svelte-ignore a11y_media_has_caption -->
      <video
        bind:this={heroEl}
        class="hero"
        {src}
        autoplay
        playsinline
        draggable="false"
        onclick={toggle}
        onplay={() => (playing = true)}
        onpause={() => (playing = false)}
        onended={() => (playing = false)}
        onloadedmetadata={() => (dur = (heroEl as HTMLVideoElement)?.duration ?? 0)}
        ontimeupdate={() => (cur = (heroEl as HTMLVideoElement)?.currentTime ?? 0)}
      ></video>
    {:else if kind === "audio"}
      <div class="audio-stage" bind:this={audioWrapEl} onclick={(e) => e.stopPropagation()}>
        <AudioPlayer {src} {name} />
      </div>
    {:else}
      <img bind:this={heroEl} class="hero" {src} alt="" draggable="false" />
    {/if}
  </div>

  {#if kind === "video"}
    <div class="chrome" bind:this={chromeEl} onclick={(e) => e.stopPropagation()}>
      <button class="pp" onclick={toggle} aria-label={playing ? "Pause" : "Play"}>
        {#if playing}
          <svg viewBox="0 0 24 24" width="22" height="22" fill="currentColor">
            <rect x="6" y="5" width="4" height="14" rx="1" />
            <rect x="14" y="5" width="4" height="14" rx="1" />
          </svg>
        {:else}
          <svg viewBox="0 0 24 24" width="22" height="22" fill="currentColor">
            <path d="M8 5v14l11-7z" />
          </svg>
        {/if}
      </button>
      <div
        class="seek"
        onpointerdown={seek}
        role="slider"
        aria-label="Seek"
        aria-valuenow={Math.round(pct)}
        aria-valuemin="0"
        aria-valuemax="100"
        tabindex="-1"
      >
        <span class="fill" style="width:{pct}%"></span>
      </div>
      <span class="time">{fmt(cur)} / {fmt(dur)}</span>
    </div>
  {/if}
</div>

<style>
  .lightbox {
    position: fixed;
    inset: 0;
    z-index: 100;
    opacity: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 48px;
    cursor: zoom-out;
  }
  .backdrop {
    position: absolute;
    inset: 0;
    background: rgba(40, 38, 32, 0.26);
    -webkit-backdrop-filter: blur(14px) saturate(140%);
    backdrop-filter: blur(14px) saturate(140%);
  }

  .stage {
    position: relative;
    flex: 1 1 auto;
    min-height: 0;
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .hero {
    max-width: min(92vw, 1400px);
    max-height: 100%;
    object-fit: contain;
    border-radius: 18px;
    box-shadow:
      0 24px 60px rgba(0, 0, 0, 0.36),
      0 0 0 1px rgba(255, 255, 255, 0.12);
    transform-origin: center center;
  }
  video.hero {
    cursor: pointer;
  }
  .audio-stage {
    position: relative;
    width: min(560px, 86vw);
  }

  .chrome {
    position: relative;
    flex: 0 0 auto;
    margin-top: 22px;
    display: flex;
    align-items: center;
    gap: 14px;
    width: min(620px, 80vw);
    padding: 12px 18px;
    border-radius: 999px;
    background: rgba(20, 19, 16, 0.5);
    -webkit-backdrop-filter: blur(16px) saturate(150%);
    backdrop-filter: blur(16px) saturate(150%);
    box-shadow: 0 8px 30px rgba(0, 0, 0, 0.35);
    color: #fff;
    cursor: default;
  }
  .pp {
    flex: 0 0 auto;
    width: 38px;
    height: 38px;
    display: grid;
    place-items: center;
    border: none;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.16);
    color: #fff;
    cursor: pointer;
    transition: background 0.15s ease;
  }
  .pp:hover {
    background: rgba(255, 255, 255, 0.28);
  }
  .seek {
    position: relative;
    flex: 1 1 auto;
    height: 16px;
    display: flex;
    align-items: center;
    cursor: pointer;
    touch-action: none;
  }
  .seek::before {
    content: "";
    position: absolute;
    left: 0;
    right: 0;
    height: 4px;
    border-radius: 99px;
    background: rgba(255, 255, 255, 0.25);
  }
  .fill {
    position: relative;
    height: 4px;
    border-radius: 99px;
    background: #fff;
  }
  .time {
    flex: 0 0 auto;
    font-size: 13px;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
    opacity: 0.8;
  }
</style>
