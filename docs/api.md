# Toile — API

The boundary between the SvelteKit canvas and the Rust core: ten Tauri commands, two events,
and one loopback HTTP server. Toile exposes nothing to the network beyond `127.0.0.1`.

Everything here is read from `apps/desktop/src-tauri/src/lib.rs` and its callers in
`apps/desktop/src/lib/`.

## Commands

Called from the frontend with `invoke<T>("name", { args })` from `@tauri-apps/api/core`. Every
fallible command returns `Result<T, String>`, which surfaces as a rejected promise carrying the
error string. All of them are registered in the `tauri::generate_handler![…]` list inside
`run()`.

| Command | Arguments | Returns | What it does |
|---|---|---|---|
| `init_board` | — | `InitData` | Scans the notes folder for `.md` files, parses each, and returns them |
| `write_note` | `note: Note` | `()` | Serializes a note to `<folder>/<id>.md` and memoizes the bytes |
| `delete_note` | `id: string` | `()` | Marks the id deleted, then removes the file |
| `save_asset` | `data: number[]`, `ext: string` | `string` | Stores bytes under `assets/`, returns the relative path |
| `load_strokes` | — | `string` | Raw contents of `.toile-drawing.json`, or `"{}"` |
| `save_strokes` | `data: string` | `()` | Overwrites `.toile-drawing.json` |
| `load_links` | — | `string` | Raw contents of `.toile-links.json`, or `"{}"` |
| `save_links` | `data: string` | `()` | Overwrites `.toile-links.json` |
| `fetch_link_preview` | `url: string` | `LinkMeta` | Unfurls a URL. Async |
| `yt_port` | — | `number` | Port the loopback YouTube proxy bound to, or `0` if it failed to start |

### Payload shapes

`Note` crosses the boundary in both directions:

```ts
type Note = {
  id: string;
  x: number; y: number; w: number; h: number;
  color: string;
  z: number;
  text: string;
  font?: "sans" | "serif" | "mono";
  size?: number;
};
```

`init_board` returns:

```ts
type InitData = { folder: string; notes: Note[]; unfurl: boolean };
```

`folder` is the absolute resolved notes directory — the frontend needs it to build `asset:`
URLs in `src/lib/assets.ts`. `unfurl` mirrors the `~/.toile.yml` key so the client can skip
queueing previews entirely.

`fetch_link_preview` returns a camelCase struct with every optional field omitted when absent:

```ts
type LinkMeta = {
  url: string;
  kind: "youtube" | "card";
  title?: string;
  description?: string;   // truncated to 300 characters
  siteName?: string;
  image?: string;         // relative assets/ path, already downloaded
  favicon?: string;       // relative assets/ path, already downloaded
  accent?: string;        // the page's theme-color
  videoId?: string;       // youtube only
};
```

Note that `image` and `favicon` are **local paths, not remote URLs** — the Rust side downloads
them into `assets/` before returning. The frontend adds a third `kind`, `"error"`, for rejected
lookups; that value never comes from Rust and is never persisted to `.toile-links.json`.

### Error behavior worth knowing

- `load_strokes` and `load_links` never fail. A missing or unreadable file yields `"{}"`.
- `delete_note` ignores the filesystem result, so deleting a note that is already gone
  succeeds.
- `save_asset` sanitizes the extension to at most five lowercase alphanumerics, falling back to
  `bin`. The filename is a hash of the bytes, so writing identical content twice is a no-op.
- `fetch_link_preview` rejects with `"unsupported url scheme"`, `"missing host"`,
  `"could not resolve host"`, `"host resolved to nothing"`, `"address not allowed"`,
  `"too many redirects"` or `"server returned <status>"`.

## Events

Emitted from the Rust watcher thread through `AppHandle::emit`, so the frontend subscribes with
`listen()`.

| Event | Payload | When |
|---|---|---|
| `note-changed` | `Note` | A `.md` file in the folder was created or modified by something other than Toile |
| `note-removed` | `{ id: string }` | A `.md` file disappeared and the app did not delete it |

Both are filtered before emission. `handle_fs_event` compares the new file contents against the
`last` map that `write_note` populates and drops the event if they match; `delete_note` records
the id in a `deleted` set that suppresses exactly one matching removal. Without those two
filters every local edit would round-trip and fight the editor.

The watcher runs in `NonRecursive` mode, so nothing under `assets/` is reported.

## Loopback YouTube proxy

`start_yt_proxy()` binds a `tiny_http` server to `127.0.0.1:0` at startup and spawns a thread
for it. The frontend asks for the port with `yt_port` and builds iframe URLs against it.

| Method | Path | Response |
|---|---|---|
| `GET` | `/yt/<video_id>` | `text/html` page whose only content is a full-bleed iframe onto `https://www.youtube-nocookie.com/embed/<id>?autoplay=1&rel=0&playsinline=1` |
| any | anything else | `400 bad request` |

`<video_id>` must be exactly 11 characters of `[A-Za-z0-9_-]`; anything else is rejected before
the page is built. Query strings and fragments are stripped from the path first.

If the bind fails the app still starts and `yt_port` returns `0` — YouTube cards then have
nowhere to point. The port is ephemeral and changes on every launch, which is why it is fetched
rather than assumed.

The iframe indirection is deliberate. Embedding YouTube directly in the app's own origin makes
the player inherit the board's CSS `zoom`, and the comment in `yt_embed_page()` records that
viewport-unit sizing tricks made the player jump around under zoom. The proxy page gives the
player a plain, unzoomed origin of its own, and the card owns the aspect ratio.

## Permissions

`src-tauri/capabilities/default.json` grants the `main` window exactly four permissions:
`core:default`, `opener:default`, `opener:allow-open-path` and
`clipboard-manager:allow-read-image`. Opening a file from a note goes through the opener
plugin; pasting an image from the system clipboard needs the last one. Any new plugin call has
to be added here or it fails at runtime with a permission error rather than a missing command.
