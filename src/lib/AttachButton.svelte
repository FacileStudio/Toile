<script lang="ts">
  let { onfiles }: { onfiles: (files: File[]) => void } = $props();

  let input = $state<HTMLInputElement | null>(null);

  function pick() {
    input?.click();
  }

  function onChange(e: Event) {
    const el = e.target as HTMLInputElement;
    const files = Array.from(el.files ?? []);
    el.value = "";
    if (files.length) onfiles(files);
  }
</script>

<button
  class="attach liquid-glass"
  title="Add image"
  aria-label="Add image"
  onclick={pick}
>
  <svg
    viewBox="0 0 24 24"
    width="20"
    height="20"
    fill="none"
    stroke="currentColor"
    stroke-width="1.8"
    stroke-linecap="round"
    stroke-linejoin="round"
  >
    <path
      d="M21.44 11.05l-9.19 9.19a5 5 0 0 1-7.07-7.07l9.19-9.19a3 3 0 0 1 4.24 4.24l-9.2 9.19a1 1 0 0 1-1.41-1.41l8.49-8.49"
    />
  </svg>
</button>
<input
  bind:this={input}
  class="file-input"
  type="file"
  accept="image/*"
  multiple
  onchange={onChange}
/>

<style>
  .attach {
    width: 54px;
    height: 54px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    color: var(--ink-soft);
    cursor: pointer;
    transition:
      transform 0.2s var(--ease-soft),
      color 0.2s ease;
  }
  .attach:hover {
    transform: translateY(-3px) scale(1.06);
    color: var(--ink);
  }
  .attach:active {
    transform: translateY(-1px) scale(0.94);
  }
  .file-input {
    display: none;
  }

  @media (prefers-reduced-motion: reduce) {
    .attach {
      transition: none !important;
    }
  }
</style>
