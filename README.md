# Tableau

An infinite canvas for sticky notes. Pan, zoom, drop colored notes anywhere, and drag them off when you're done. A small native desktop app built with Tauri + SvelteKit.

Paste (`⌘V`) or drag an image onto the canvas to pin it — onto a note to add it there, or onto empty space to spawn a new note. Images are saved next to your notes under `assets/` and referenced as plain markdown (`![](assets/…)`), so your notes stay portable and Obsidian-readable.

## Develop

```bash
bun install
bun run tauri dev
```

## Build

```bash
bun run tauri build
```

Bundles land in `src-tauri/target/release/bundle/`.

## Stack

Tauri 2 · SvelteKit (Svelte 5 runes) · TypeScript

## License

MIT
