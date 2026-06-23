<script lang="ts">
  import { fade } from "svelte/transition";

  let {
    images,
    onclose,
  }: { images: string[]; onclose: () => void } = $props();
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
  class="lightbox"
  data-ui
  transition:fade={{ duration: 160 }}
  onclick={onclose}
>
  {#each images as src}
    <img class="lightbox-img" {src} alt="" draggable="false" />
  {/each}
</div>

<style>
  .lightbox {
    position: fixed;
    inset: 0;
    z-index: 100;
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
  }
</style>
