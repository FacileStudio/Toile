<script lang="ts">
  import { onMount } from "svelte";

  let {
    src,
    name,
    onfail,
  }: { src: string; name: string; onfail?: () => void } = $props();

  const VB_W = 200;
  const MID = 12;
  const LAMBDA = 34;
  const AMP = 4.5;
  const SPEED = 0.0045;

  let audio = $state<HTMLAudioElement | null>(null);
  let playing = $state(false);
  let cur = $state(0);
  let dur = $state(0);
  let objectUrl = $state<string | null>(null);
  const pct = $derived(dur > 0 ? Math.min(100, (cur / dur) * 100) : 0);

  const STEP = 3;
  let wavePath = $state(`M0 ${MID} L0 ${MID}`);
  let tipY = $state(MID);
  const thumbTop = $derived((tipY / 24) * 100);

  let phase = 0;
  let amp = 0;
  let raf = 0;
  let prev = 0;
  let lastNow = 0;
  let lastReal = 0;

  const waveAt = (x: number) => MID + amp * Math.sin((2 * Math.PI * x) / LAMBDA + phase);

  function buildWave() {
    const playX = (pct / 100) * VB_W;
    let d = `M0 ${waveAt(0).toFixed(2)}`;
    for (let x = STEP; x < playX; x += STEP) d += ` L${x.toFixed(1)} ${waveAt(x).toFixed(2)}`;
    d += ` L${playX.toFixed(2)} ${waveAt(playX).toFixed(2)}`;
    wavePath = d;
    tipY = waveAt(playX);
  }

  function render() {
    if (audio) {
      if (playing) {
        const now = performance.now();
        const dt = lastNow ? Math.min(0.25, (now - lastNow) / 1000) : 0;
        lastNow = now;
        const real = audio.currentTime;
        if (real < lastReal - 0.05) {
          cur = real;
        } else {
          let next = cur + dt * (audio.playbackRate || 1);
          if (next < real)
            next = real;
          else if (next > real + 0.5) next = real + 0.5;
          if (dur) next = Math.min(next, dur);
          cur = Math.max(cur, next);
        }
        lastReal = real;
      } else {
        cur = audio.currentTime;
        lastNow = 0;
      }
    }
    buildWave();
  }

  function frame(t: number) {
    const dt = prev ? Math.min(48, t - prev) : 16;
    prev = t;
    if (playing) phase += SPEED * dt;
    const target = playing ? AMP : 0;
    amp += (target - amp) * Math.min(1, dt / 120);
    render();
    if (playing || Math.abs(amp - target) > 0.02) {
      raf = requestAnimationFrame(frame);
    } else {
      amp = target;
      render();
      raf = 0;
      prev = 0;
    }
  }

  function kick() {
    if (!raf) {
      prev = 0;
      raf = requestAnimationFrame(frame);
    }
  }

  $effect(() => {
    playing;
    kick();
  });

  $effect(() => {
    pct;
    if (!raf) buildWave();
  });

  $effect(() => {
    let url: string | null = null;
    let cancelled = false;
    fetch(src)
      .then((r) => r.blob())
      .then((b) => {
        if (cancelled) return;
        url = URL.createObjectURL(b);
        objectUrl = url;
      })
      .catch(() => onfail?.());
    return () => {
      cancelled = true;
      if (url) URL.revokeObjectURL(url);
      objectUrl = null;
    };
  });

  function toggle() {
    const a = audio;
    if (!a || !objectUrl) return;
    if (a.paused) a.play().catch(() => onfail?.());
    else a.pause();
  }

  function seek(e: PointerEvent) {
    const a = audio;
    if (!a || !dur) return;
    const r = (e.currentTarget as HTMLElement).getBoundingClientRect();
    a.currentTime = Math.max(0, Math.min(1, (e.clientX - r.left) / r.width)) * dur;
    cur = a.currentTime;
    lastNow = 0;
    if (!raf) buildWave();
  }

  function fmt(t: number): string {
    if (!isFinite(t) || t < 0) return "0:00";
    const m = Math.floor(t / 60);
    const s = Math.floor(t % 60);
    return `${m}:${s.toString().padStart(2, "0")}`;
  }

  onMount(() => () => {
    if (raf) cancelAnimationFrame(raf);
  });
</script>

<div class="player note-audio" data-src={src} data-name={name}>
  <div class="row">
    <button
      class="play audio-ctl"
      onclick={toggle}
      aria-label={playing ? "Pause" : "Play"}
    >
      {#if playing}
        <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
          <rect x="6" y="5" width="4" height="14" rx="1" />
          <rect x="14" y="5" width="4" height="14" rx="1" />
        </svg>
      {:else}
        <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
          <path d="M8 5v14l11-7z" />
        </svg>
      {/if}
    </button>
    <span class="name">{name}</span>
    <span class="time">{fmt(cur)}</span>
  </div>

  <div
    class="bar audio-ctl"
    onpointerdown={seek}
    role="slider"
    aria-label="Seek"
    aria-valuenow={Math.round(pct)}
    aria-valuemin="0"
    aria-valuemax="100"
    tabindex="-1"
  >
    <span class="track" style="clip-path: inset(0 0 0 calc({pct}% + 4px))"></span>
    <div class="fg">
      <svg class="wave" viewBox="0 0 {VB_W} 24" preserveAspectRatio="none">
        <path d={wavePath} vector-effect="non-scaling-stroke" />
      </svg>
    </div>
    <span class="thumb" style="left:{pct}%; top:{thumbTop}%"></span>
  </div>

  <audio
    bind:this={audio}
    src={objectUrl}
    onplay={() => {
      playing = true;
      lastNow = 0;
    }}
    onpause={() => {
      playing = false;
      if (audio) cur = audio.currentTime;
    }}
    onended={() => {
      playing = false;
      cur = dur;
    }}
    onloadedmetadata={() => (dur = audio?.duration ?? 0)}
    ondurationchange={() => (dur = audio?.duration ?? 0)}
    onerror={() => onfail?.()}
  ></audio>
</div>

<style>
  .player {
    display: flex;
    flex-direction: column;
    gap: 13px;
    width: 100%;
    box-sizing: border-box;
    margin: 6px 0;
    padding: 16px;
    border-radius: 15px;
    background: rgba(255, 255, 255, 0.55);
    box-shadow: inset 0 0 0 1px rgba(40, 38, 32, 0.12);
    color: var(--ink);
  }

  .row {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
  }
  .play {
    flex: 0 0 auto;
    width: 40px;
    height: 40px;
    display: grid;
    place-items: center;
    border: none;
    border-radius: 50%;
    background: var(--ink);
    color: #fff;
    cursor: pointer;
    transition: transform 0.15s var(--ease-soft);
  }
  .play:hover {
    transform: scale(1.07);
  }
  .play:active {
    transform: scale(0.92);
  }
  .name {
    flex: 1 1 auto;
    min-width: 0;
    font-size: 17px;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .time {
    flex: 0 0 auto;
    font-size: 13px;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
    opacity: 0.55;
  }

  .bar {
    position: relative;
    height: 26px;
    cursor: pointer;
    touch-action: none;
  }
  .track {
    position: absolute;
    left: 0;
    right: 0;
    top: 50%;
    height: 3px;
    transform: translateY(-50%);
    border-radius: 99px;
    background: rgba(40, 38, 32, 0.22);
  }
  .fg {
    position: absolute;
    inset: 0;
  }
  .wave {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    overflow: visible;
    fill: none;
    stroke: var(--ink);
    stroke-width: 2.4;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .thumb {
    position: absolute;
    width: 11px;
    height: 11px;
    margin: -5.5px 0 0 -5.5px;
    border-radius: 50%;
    background: var(--ink);
    box-shadow: 0 1px 3px rgba(40, 38, 32, 0.3);
    pointer-events: none;
  }
</style>
