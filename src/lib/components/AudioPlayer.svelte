<script lang="ts">
  let {
    src,
    name,
    onfail,
  }: { src: string; name: string; onfail?: () => void } = $props();

  // A Material-3-style wavy scrubber: the played portion is a sine wave that
  // travels while playing and flattens (scaleY → ~0) when paused. The path is
  // periodic over LAMBDA so the travel loop is seamless; it's drawn one extra
  // wavelength wide so the leftward shift never reveals a gap.
  const LAMBDA = 24;
  const AMP = 6;
  const MID = 12;
  const VB_W = 200;

  const wavePath = (() => {
    let d = `M0 ${MID}`;
    for (let x = 0; x <= VB_W + LAMBDA; x += 3) {
      const y = MID + AMP * Math.sin((2 * Math.PI * x) / LAMBDA);
      d += ` L${x} ${y.toFixed(2)}`;
    }
    return d;
  })();

  let audio = $state<HTMLAudioElement | null>(null);
  let playing = $state(false);
  let cur = $state(0);
  let dur = $state(0);

  const pct = $derived(dur > 0 ? Math.min(100, (cur / dur) * 100) : 0);

  function toggle() {
    const a = audio;
    if (!a) return;
    if (a.paused) a.play().catch(() => onfail?.());
    else a.pause();
  }

  function seek(e: PointerEvent) {
    const a = audio;
    if (!a || !dur) return;
    const r = (e.currentTarget as HTMLElement).getBoundingClientRect();
    a.currentTime = Math.max(0, Math.min(1, (e.clientX - r.left) / r.width)) * dur;
  }

  function fmt(t: number): string {
    if (!isFinite(t) || t < 0) return "0:00";
    const m = Math.floor(t / 60);
    const s = Math.floor(t % 60);
    return `${m}:${s.toString().padStart(2, "0")}`;
  }
</script>

<div class="player note-audio" style="--amp:{playing ? 1 : 0.14}">
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
    class:playing
    onpointerdown={seek}
    role="slider"
    aria-label="Seek"
    aria-valuenow={Math.round(pct)}
    aria-valuemin="0"
    aria-valuemax="100"
    tabindex="-1"
  >
    <span class="track"></span>
    <div class="fg" style="clip-path: inset(0 {100 - pct}% 0 0)">
      <svg class="wave" viewBox="0 0 {VB_W} 24" preserveAspectRatio="none">
        <g class="amp">
          <g class="travel">
            <path d={wavePath} vector-effect="non-scaling-stroke" />
          </g>
        </g>
      </svg>
    </div>
    <span class="thumb" style="left:{pct}%"></span>
  </div>

  <audio
    bind:this={audio}
    {src}
    preload="metadata"
    onplay={() => (playing = true)}
    onpause={() => (playing = false)}
    onended={() => (playing = false)}
    onloadedmetadata={() => (dur = audio?.duration ?? 0)}
    ontimeupdate={() => (cur = audio?.currentTime ?? 0)}
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
    background: rgba(40, 38, 32, 0.2);
  }
  .fg {
    position: absolute;
    inset: 0;
    overflow: hidden;
  }
  .wave {
    width: 100%;
    height: 100%;
    fill: none;
    stroke: var(--ink);
    stroke-width: 2.4;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .amp {
    transform: scaleY(var(--amp));
    transform-box: fill-box;
    transform-origin: center;
    transition: transform 0.4s cubic-bezier(0.22, 1, 0.36, 1);
  }
  .travel {
    transform-box: fill-box;
    animation: travel 1.15s linear infinite;
    animation-play-state: paused;
  }
  .bar.playing .travel {
    animation-play-state: running;
  }
  @keyframes travel {
    from {
      transform: translateX(0);
    }
    to {
      transform: translateX(-24px);
    }
  }
  .thumb {
    position: absolute;
    top: 50%;
    width: 11px;
    height: 11px;
    margin-left: -5.5px;
    transform: translateY(-50%);
    border-radius: 50%;
    background: var(--ink);
    box-shadow: 0 1px 3px rgba(40, 38, 32, 0.3);
    pointer-events: none;
  }

  @media (prefers-reduced-motion: reduce) {
    .travel {
      animation: none;
    }
    .amp {
      transition: none;
    }
  }
</style>
