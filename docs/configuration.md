# Toile — Configuration

Everything that changes Toile's behavior: one YAML file in your home directory, a handful of
compiled-in constants, and the Tauri window definition.

## `~/.toile.yml`

The only user-facing configuration. If the file does not exist when the app starts,
`load_config_folder()` writes this template and uses `~/Toile`:

```yaml
# Toile — where your notes live as markdown files.
# Point this at any folder. To read your notes in Obsidian on your phone,
# set it to a folder inside an iCloud Obsidian vault, e.g.:
# folder: ~/Library/Mobile Documents/iCloud~md~obsidian/Documents/MyVault/Toile
#
# Every .md file in this folder becomes a postit. A note created by hand
# (no frontmatter, just text) shows up automatically; Toile only writes
# layout frontmatter once you move or restyle it in the app.
folder: /Users/you/Toile
```

| Key | Required | Default | What it does |
|---|---|---|---|
| `folder` | no | `~/Toile` | Directory holding the `.md` notes, the `assets/` subfolder and the two sidecars. Created on startup if missing |
| `unfurl` | no | `true` | When `false`, `fetch_link_preview` is never called, so pasted links stay plain and the app makes no outbound requests |

Both keys are read independently and both fall back on any parse problem: a malformed file, a
missing key, or an empty `folder` string all resolve to the defaults rather than erroring.

`~` expansion is deliberate but narrow — `expand_tilde()` handles a bare `~` and a leading `~/`
only. A path like `~otheruser/notes` is taken literally and will not resolve.

`folder` is read exactly once, during `setup()`. Editing it while Toile is running has no
effect until you restart, because the folder path and the filesystem watcher are both captured
in the shared `SyncState` at startup.

## What Toile writes into the notes folder

| Path | Written by | Contents |
|---|---|---|
| `<id>.md` | `write_note` | One note: YAML frontmatter (`x`, `y`, `w`, `h`, `color`, `z`, optional `font` and `size`) then the body |
| `assets/<hash>.<ext>` | `save_asset`, `download_asset` | Pasted or dropped files, and downloaded link-preview covers and favicons. Named by a hash of the bytes, so identical content is stored once |
| `.toile-drawing.json` | `save_strokes` | The freehand stroke list, debounced 400 ms |
| `.toile-links.json` | `save_links` | The link-preview cache. Entries whose `kind` is `error` are stripped before saving |

The watcher is non-recursive, so changes inside `assets/` do not trigger note reloads. The two
sidecars are ignored by the note scanner because it only accepts a `.md` extension.

## Limits compiled into the app

None of these are configurable without rebuilding.

| Constant | Value | Where | What it caps |
|---|---|---|---|
| `MAX_ASSET_BYTES` | 100 MB | `src/routes/+page.svelte` | Largest file you can drop or paste. Bigger files raise a toast and are skipped |
| `MAX_IMG` | 8 MB | `src-tauri/src/lib.rs` | Largest link-preview image or favicon downloaded |
| `MAX_HTML` | 1.5 MB | `src-tauri/src/lib.rs` | How much of a page is parsed for metadata |
| Link fetch timeout | 12 s | `src-tauri/src/lib.rs` | Per-request timeout on the preview client |
| Concurrent unfurls | 4 | `src/lib/links.svelte.ts` | Queue width for link previews |
| `MIN_SCALE` / `MAX_SCALE` | 0.05 / 3 | `src/lib/board.svelte.ts` | Zoom range |
| `NOTE_SIZE` | 224 | `src/lib/board.svelte.ts` | Default note box, in world units |
| `TEXT_WIDTH` / `MEDIA_WIDTH` | 260 / 380 | `src/lib/board.svelte.ts` | Default width for a text note and a media-only note |
| `TEXT_SIZE` | 12 – 80, default 19 | `src/lib/board.svelte.ts` | Per-note text size range |
| `PALETTE` | 6 colors | both sides | `#ffe8a3`, `#ffc9c9`, `#c9e8ca`, `#bfe0f2`, `#e2cdf2`, `#ffd6b0` |

The palette is duplicated in `lib.rs` and `board.svelte.ts`. Changing one without the other
makes hand-created notes pick colors the in-app swatches cannot reproduce.

## Environment variables

Toile reads no environment variables at runtime. One is read at build time:

| Variable | Required | Default | What it does |
|---|---|---|---|
| `TAURI_DEV_HOST` | no | unset | Set by `tauri dev` when targeting a device on the network. `vite.config.js` binds the dev server to it and points HMR at port 1421 over `ws` |

The dev server is pinned to port 1420 with `strictPort: true`, because `tauri.conf.json`
hardcodes `devUrl: "http://localhost:1420"`. If that port is taken, the dev build fails rather
than silently moving.

## Window and bundle

From `src-tauri/tauri.conf.json`:

- Product name `Toile`, identifier `dev.saravenpi.toile`, version `0.1.0`.
- Window 1280 by 832, minimum 640 by 480, background `#f7f6f2`, overlay title bar with a
  hidden title.
- `dragDropEnabled: false` — the WebView handles drops itself, so Tauri's native handler must
  stay out of the way.
- `bundle.targets: "all"`, icons from `src-tauri/icons/`.
- The frontend is prebuilt: `beforeBuildCommand` runs `bun run build` and `frontendDist` points
  at `../build`, which `@sveltejs/adapter-static` fills with an SPA fallback `index.html`.

### Content security

The CSP allows `'unsafe-inline'` and `'unsafe-eval'`, `asset:` plus `https:` for images and
media, and limits `frame-src` to localhost and `youtube.com` / `youtube-nocookie.com`.

The asset protocol is enabled with `scope: ["$HOME/**"]`. That is broader than the notes folder:
any file under the user's home directory can be loaded into the WebView through
`convertFileSrc`. It has to cover `$HOME` because `folder` can point anywhere, but it is worth
knowing before you paste a markdown link with an absolute path into a note.

## Client-side state

The camera position (pan and zoom) is not part of the notes folder. It is stored in the
WebView's `localStorage` under the key `toile.camera.v1`, so it is per-machine and does not
follow a synced vault.
