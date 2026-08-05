# Toile

A calm infinite canvas where your notes live as plain markdown files on your own disk.

Every sticky note on the board is one `.md` file in a folder you choose, with its position and
color kept in YAML frontmatter. Point that folder at an Obsidian vault and the same notes open
on your phone. The repo is a two-app monorepo: the Tauri desktop app, and the static landing
page published at [toile.facile.studio](https://toile.facile.studio).

## What it does

- Infinite pan-and-zoom canvas of colored sticky notes, drawings, media and link cards
- Stores every note as a plain markdown file — layout in frontmatter, body as you typed it
- Watches the notes folder and reflects edits made outside the app, live, without a reload
- Freehand drawing layer with an eraser, kept in a JSON sidecar next to the notes
- Paste or drop images, video, audio and files: they are copied into `assets/` and linked as
  ordinary markdown, so the folder stays portable
- Unfurls pasted links into cards with title, description, favicon and cover, and plays
  YouTube links inline through a loopback proxy
- Renders GFM markdown, including task checkboxes you can tick straight on the note
- Undo and redo, multi-select, resize, per-note font and text size, minimap, and a `⌘K` search

## Stack

| Layer | Tech |
|---|---|
| Desktop | Tauri 2, Rust 2021, `notify` file watcher, `reqwest` + `scraper` for link previews |
| Client | SvelteKit 2 (Svelte 5 runes), TypeScript 5.6, `adapter-static` in SPA mode, `marked` 18 |
| Storage | Markdown files plus JSON sidecars in one folder — no database, no server |
| Deploy | `apps/web` only: static HTML in an `nginx:alpine` container |

## Quick start

The desktop app needs [Bun](https://bun.sh) and a Rust toolchain.

```sh
cd apps/desktop
bun install
bun run tauri dev
```

On first run Toile writes `~/.toile.yml` and creates `~/Toile` as the notes folder. Press `?`
inside the app for the keyboard shortcuts.

To produce installers:

```sh
bun run tauri build
```

Bundles land in `apps/desktop/src-tauri/target/release/bundle/`.

## Configuration

Toile reads no environment variables. Everything lives in `~/.toile.yml`, written with a
commented template the first time the app starts.

| Key | What it does |
|---|---|
| `folder` | Where notes and `assets/` live. `~` is expanded. Defaults to `~/Toile` |
| `unfurl` | Set to `false` to stop fetching link previews from the network. Defaults to `true` |

Full reference: [docs/configuration.md](docs/configuration.md).

## Structure

```
apps/
  desktop/   Tauri app — src/ (SvelteKit canvas), src-tauri/ (Rust commands, watcher)
  web/       Landing page — a single index.html plus an nginx Dockerfile
docs/        Architecture, configuration, development, IPC surface
```

## Documentation

| Doc | What's in it |
|---|---|
| [Architecture](docs/architecture.md) | Process topology, the file format, how sync works |
| [Configuration](docs/configuration.md) | `~/.toile.yml`, the files Toile writes, window settings |
| [Development](docs/development.md) | Local setup, the check gate, building and releasing |
| [API](docs/api.md) | Tauri commands, events, and the loopback YouTube proxy |

---

Part of the [Facile Suite](https://facile.studio) — self-hosted tools for creative studios
and freelancers. One login, zero cloud dependency.
