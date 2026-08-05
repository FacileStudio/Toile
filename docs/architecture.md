# Toile — Architecture

How the desktop app is put together, what a note actually is on disk, and how the canvas and
the filesystem stay in sync.

## Runtime topology

```
┌─ Toile.app (Tauri 2) ──────────────────────────────────────────────┐
│                                                                    │
│  WebView (SvelteKit SPA)          Rust core (toile_lib)            │
│    board.svelte.ts  ──invoke──▶  10 #[tauri::command] handlers      │
│    links.svelte.ts  ◀──emit────  note-changed / note-removed        │
│    drawing.svelte.ts                    │                           │
│         │                               │                           │
│         │ asset: protocol                │ std::fs                   │
│         └───────────────────┬────────────┘                           │
│                             ▼                                        │
│                   ~/Toile  (or `folder` from ~/.toile.yml)           │
│                     <id>.md              one file per note           │
│                     assets/              images, video, audio        │
│                     .toile-drawing.json  strokes                     │
│                     .toile-links.json    link preview cache          │
│                             ▲                                        │
│                    notify watcher (non-recursive)                    │
│                                                                      │
│  tiny_http on 127.0.0.1:<random>  ──▶ /yt/<id> ──▶ youtube-nocookie  │
└──────────────────────────────────────────────────────────────────────┘
```

There is no backend service and no network listener reachable from outside the machine. The
only always-on socket is the loopback YouTube proxy, bound to `127.0.0.1:0` so the OS picks a
free ephemeral port.

## Components

**`apps/desktop/src-tauri/src/lib.rs`** is the whole Rust side — about 830 lines holding the
command handlers, the markdown-with-frontmatter codec, the filesystem watcher, the link
unfurler and the YouTube proxy. `main.rs` does nothing but call `toile_lib::run()`.

**`apps/desktop/src/routes/+page.svelte`** is the canvas: pointer handling, selection, panning,
zooming, keyboard shortcuts, paste and drop. Everything else in `src/lib/components/` is a
widget it composes — `Postit`, `TextBlock`, `LinkCard`, `AudioPlayer`, `VideoPlayer`,
`Minimap`, `SearchPalette`, `Cheatsheet`, `Palette`, `Trash`, `UndoRedo`, `ZoomControls`.

**State lives in three Svelte 5 rune classes** under `src/lib/`:

| Module | Holds |
|---|---|
| `board.svelte.ts` | Notes, strokes, camera, z-order, and the undo/redo stack |
| `drawing.svelte.ts` | The active pen: color, width, in-progress path |
| `links.svelte.ts` | The link-preview cache, with a four-at-a-time fetch queue |

**`apps/web/`** is unrelated at runtime: one 489-line `index.html` with inline CSS, and a
`Dockerfile` that copies the directory into `nginx:alpine` and serves `/downloads/` with a
forced `Content-Disposition: attachment`. The released `Toile-aarch64.dmg` and `Toile-x64.dmg`
are committed to the repository under `apps/web/downloads/`, as are the Goga OTF files, so the
image needs no build artifacts from outside git.

## What a note is

One `.md` file per note, named `<id>.md`, where the id is the file stem. Toile writes YAML
frontmatter for layout and leaves the body exactly as typed:

```markdown
---
x: -120.0
y: 88.0
w: 224.0
h: 224.0
color: '#ffe8a3'
z: 7.0
---
Buy milk
- [ ] and bread
```

Every frontmatter key is optional on read (`PartialFm` in `lib.rs`). A file dropped into the
folder by hand — no frontmatter, just text — becomes a note immediately: `build_note()` hashes
the id to derive a stable pseudo-random position within roughly ±450 by ±350, picks one of six
palette colors from the same hash, and defaults the box to 224 by 224. Toile only writes
frontmatter back once you move or restyle the note in the app. That is what keeps a shared
Obsidian vault readable.

Two optional keys carry styling: `font` (`sans`, `serif` or `mono`, resolved to stacks in
`src/lib/fonts.ts`) and `size` (text size, 12 to 80, default 19).

Attachments are markdown links to `assets/<hash>.<ext>` — `![](…)` for media, `[](…)` for
other files. `save_asset` names files by a hash of their bytes, so re-pasting the same image
does not duplicate it. `src/lib/assets.ts` classifies an extension as image, video, audio or
file, and `src/lib/segments.ts` splits a note body into renderable segments.

Two sidecars sit beside the notes and are ignored by the note scanner because they are not
`.md`: `.toile-drawing.json` (the stroke list) and `.toile-links.json` (the unfurl cache).

## Sync lifecycle

1. On startup `setup()` reads `~/.toile.yml`, creates the folder, builds the shared
   `SyncState` (folder, `unfurl` flag, a `last`-content map, a `deleted` set and the top
   z-index), and spawns a `notify` watcher on the folder in `NonRecursive` mode.
2. The frontend calls `init_board`, which scans every `.md` file in the folder, parses it, and
   returns the notes plus the folder path and the `unfurl` flag. The camera position is
   restored separately from `localStorage` under `toile.camera.v1`.
3. Editing a note calls `write_note`, which serializes it and records the exact bytes in
   `state.last` before writing.
4. The watcher fires on every change. `handle_fs_event` skips non-`.md` paths, re-reads the
   file, and compares it against `state.last` — if identical, the event was Toile's own write
   and is dropped. Otherwise it emits `note-changed` with the reparsed note.
5. A deletion emits `note-removed`, unless the id is in the `deleted` set, which `delete_note`
   populates so the app's own deletions do not echo back.

This last-write memo is the entire conflict story: there is no locking and no merge. A note
edited simultaneously in Toile and in another editor resolves to whichever write landed last.

## Undo and redo

`board.svelte.ts` keeps two stacks of `UndoOp` values covering stroke add, stroke erase, note
add, note delete, resize and move. Operations are inverted rather than snapshotted, so undo
replays a delta. Any new operation clears the redo stack. Stroke saves are debounced 400 ms;
link-cache saves likewise.

## Link previews

Pasting a bare URL creates a card. `links.svelte.ts` looks in its cache, and on a miss queues
`fetch_link_preview` with at most four concurrent fetches. On the Rust side:

- `validate()` parses the URL, rejects anything that is not `http`/`https`, resolves the host
  and refuses loopback, private, link-local, multicast, unspecified, broadcast and
  documentation addresses. This is an SSRF guard and it runs again on every redirect hop.
- The client sends a fixed `Toile-LinkPreview/1.0` user agent, times out after 12 seconds, and
  follows redirects manually up to a cap so each hop is re-validated.
- HTML is truncated to 1.5 MB and parsed with `scraper` for Open Graph, Twitter card and
  `<title>` metadata, plus a favicon from `link[rel~=icon]` or `/favicon.ico`.
- The cover image and favicon are downloaded and stored in `assets/` through the same
  content-hash path as pasted files, capped at 8 MB each. Preview images therefore become
  permanent local files.
- Failed lookups are cached in memory as `kind: "error"` but filtered out before the cache is
  persisted, so they are retried on the next launch.

YouTube URLs skip all of that. `youtube_id()` recognizes `youtube.com`, `m.`, `music.`,
`youtube-nocookie.com` and `youtu.be`, including `/shorts/`, `/embed/`, `/v/` and `/live/`
paths, validates the 11-character id, and fetches only the title from YouTube's oEmbed
endpoint. Playback goes through the loopback proxy, which serves a minimal HTML page embedding
`youtube-nocookie.com` in an iframe. The indirection exists so the embed sits in its own
origin rather than the app's.

## Security posture

The Tauri CSP allows `asset:` and `https:` for images and media, and restricts `frame-src` to
localhost and the two YouTube hosts. The asset protocol is enabled with scope `$HOME/**`, which
means the WebView can read any file under the user's home directory through `convertFileSrc`,
not just the notes folder. `dragDropEnabled` is `false` in `tauri.conf.json` because the app
implements its own drop handling in the WebView.

Nothing in Toile authenticates, calls a Facile service, federates to Authentik, or emits
`pool` / `enveloppe` events. Outbound traffic happens only when unfurling a link, and setting
`unfurl: false` in `~/.toile.yml` removes even that.
