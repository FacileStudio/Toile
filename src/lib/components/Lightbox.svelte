<script lang="ts">
  import { onMount } from "svelte";

  let {
    src,
    kind = "image",
    origin = null,
    onclose,
  }: {
    src: string;
    kind?: "image" | "video";
    origin?: DOMRect | null;
    onclose: () => void;
  } = $props();

  const DUR = 340;
  const EASE = "cubic-bezier(0.22, 1, 0.36, 1)";
  const BLUR = "blur(22px) saturate(160%)";
  const CLEAR = "blur(0px) saturate(100%)";

  let rootEl = $state<HTMLDivElement | null>(null);
  let heroEl = $state<HTMLImageElement | HTMLVideoElement | null>(null);
  let closing = false;

  // Transform that maps the laid-out fullscreen hero back onto its on-canvas
  // origin rect (translate center->center, scale to match). Measured live, so
  // it must run only once the hero has real dimensions.
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

  async function ready() {
    if (heroEl instanceof HTMLImageElement) {
      try {
        await heroEl.decode();
      } catch {
        /* not decodable — fall back to a plain fade */
      }
    } else if (heroEl instanceof HTMLVideoElement && heroEl.readyState < 1) {
      await new Promise<void>((r) =>
        heroEl!.addEventListener("loadedmetadata", () => r(), { once: true }),
      );
    }
  }

  function blurKeyframes(a: string, b: string) {
    return [
      { backdropFilter: a, webkitBackdropFilter: a },
      { backdropFilter: b, webkitBackdropFilter: b },
    ] as Keyframe[];
  }

  function animateOpen() {
    if (!rootEl) return;
    rootEl.style.opacity = "1";
    rootEl.animate([{ opacity: 0 }, { opacity: 1 }], { duration: DUR, easing: EASE });
    rootEl.animate(blurKeyframes(CLEAR, BLUR), { duration: DUR, easing: EASE });
    const from = originTransform();
    if (from && heroEl) {
      heroEl.animate([{ transform: from }, { transform: "none" }], {
        duration: DUR,
        easing: EASE,
      });
    }
  }

  function close() {
    if (closing) return;
    closing = true;
    if (!rootEl) {
      onclose();
      return;
    }
    const to = originTransform();
    rootEl.animate(blurKeyframes(BLUR, CLEAR), {
      duration: DUR,
      easing: EASE,
      fill: "forwards",
    });
    const bg = rootEl.animate([{ opacity: 1 }, { opacity: 0 }], {
      duration: DUR,
      easing: EASE,
      fill: "forwards",
    });
    if (to && heroEl) {
      heroEl.animate([{ transform: "none" }, { transform: to }], {
        duration: DUR,
        easing: EASE,
        fill: "forwards",
      });
    }
    bg.onfinish = () => onclose();
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.stopPropagation();
      close();
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
  {#if kind === "video"}
    <!-- svelte-ignore a11y_media_has_caption -->
    <video
      bind:this={heroEl}
      class="lightbox-img"
      {src}
      controls
      autoplay
      onclick={(e) => e.stopPropagation()}
    ></video>
  {:else}
    <img bind:this={heroEl} class="lightbox-img" {src} alt="" draggable="false" />
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
    gap: 18px;
    padding: 56px;
    background: rgba(40, 38, 32, 0.32);
    -webkit-backdrop-filter: blur(22px) saturate(160%);
    backdrop-filter: blur(22px) saturate(160%);
    cursor: zoom-out;
    overflow: auto;
  }
  .lightbox-img {
    max-width: min(92vw, 1400px);
    max-height: 86vh;
    object-fit: contain;
    border-radius: 18px;
    box-shadow:
      0 30px 80px rgba(0, 0, 0, 0.45),
      0 0 0 1px rgba(255, 255, 255, 0.12);
    transform-origin: center center;
  }
</style>
