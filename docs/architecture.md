# Architecture

How a request moves through the Toile API, who may see a board, and what the database holds.
The v1 target (rooms, sharing, export, integrations) is in [plan.md](plan.md); this page
describes what the code does today.

```
Internet ──▶ Traefik ──▶ api (:4000, one replica) ─┬──▶ /health, /ready, /version, /docs
                                                   ├──▶ /api/*          auth + modules
                                                   └──▶ everything else the SPA (apps/client/build)
                                                              │
                                                         Postgres 16
```

## Components

| Path | Role |
|---|---|
| `src/server.rs` | Boot: env, logger, pool, `migrate` subcommand, migrations, kits, loops, router, drain |
| `src/auth/` | porte wiring: sessions, Registre sign-in, the spaces store, directory apply, GDPR erase |
| `src/modules/boards/` | Board CRUD, duplicate, trash, restore, purge loop, thumbnails, and the one `access()` function |
| `src/modules/board/` | The board document: schema 1 validator, seed state, `GET /api/board-schema` |
| `src/modules/rooms/` | Live rooms: hub, room task, y-websocket sync, presence, storage, drain |
| `src/modules/share/` | Share links and the guest routes |
| `src/modules/comments/` | Comment threads and their room notice |
| `src/modules/spaces/` | `GET /api/spaces`: the caller's spaces and roles |
| `src/apiref.rs` | The OpenAPI document served at `/docs` |
| `src/security.rs` | The CSP script hashes of the built client |

## Access

Every board route calls `access::access(state, user, board)`:

- a personal board (`space_id IS NULL`) is `owner` for its owner and invisible to everyone else;
- a space board is `editor` for a `member`, `owner` for an `admin` or `owner`, invisible
  otherwise;
- a share link gives `viewer`, `commenter` or `editor` to whoever holds its token;
- a deleted board is invisible; an instance admin gets no bypass; invisible always answers `404`,
  never `403`.

A space member is matched by Registre subject: `space_members.subject` joins
`porte_identities.subject` where `provider` is `OIDC_ISSUER`. Lists resolve the scope with
`access::scope` and filter in the same SQL statement.

`access::lookup(.., trashed: true)` is the same query over the trash, used only by restore and
permanent delete.

## Rooms

One room task per open board (`rooms/hub.rs`), in memory, so the app runs one replica. A room
loads `board_state.snapshot` plus every `board_updates` row, oldest first.

- Each update from an editor is applied, then checked: `board/schema.rs` watches `shapes` with a
  deep observer and validates only the shapes the transaction touched, plus the roots, `meta` and
  the shape count. A failing update is undone by rebuilding the document from the stored
  snapshot and the accepted updates since, and its sender is closed with 1008.
- Accepted updates are broadcast at once, then appended to `board_updates` after 250 ms. Every
  200 pending updates, 4 MiB or 60 s, and when the last member leaves, the room writes a snapshot
  and deletes the pending rows in one transaction.
- Server writes (`Hub::write`) and reads (`Hub::read`) go through the room, so REST, MCP and
  import changes reach open clients and validate the same way.
- An empty room unloads after 60 s. On SIGTERM every room snapshots and closes its members with
  1013 before the HTTP drain.
- Revocation: a socket re-checks its session or share link every 30 s; revoking a link, trashing
  a board, a directory snapshot that drops a member or a space, and a GDPR erase close the
  affected sockets at once with 1008.

## Directory

With `OIDC_ISSUER` set, porte's `Syncer` pulls Registre's directory snapshot (and Registre nudges
`POST /api/registre/sync`). `auth::SpacesApply` replaces `spaces` and `space_members` in one
transaction, once per snapshot version. A space missing from a snapshot is marked deleted and its
boards with it; an empty snapshot is refused while spaces exist.

`POST /api/registre/deletions` and the deletion loop erase a person: their personal boards,
their memberships, then their `porte_users` row (their comments go with it), and close their
sockets. Space boards stay with the space.

## Data model

| Table | Holds |
|---|---|
| `porte_*` | porte's schema (`0001_porte.sql`): users, identities, sessions, login codes |
| `spaces`, `space_members` | The Registre directory, by subject |
| `directory_state`, `deletion_state` | The directory version and the deletion cursor |
| `boards` | `id`, `owner_user_id`, `space_id` (null for personal), `name`, `created_at`, `updated_at`, `deleted_at` (the trash), `thumbnail`, `thumbnail_at` |
| `board_state` | The last snapshot of each board (a Yjs v1 update) and its version |
| `board_updates` | Updates accepted since that snapshot, oldest first |
| `share_links` | `token_hash` (SHA-256 of the token), `role` view, comment or edit, `revoked_at` |
| `comments` | Threads (`parent_id`), `anchor` jsonb, `body`, the author user or guest name, `resolved_at` |

## Client

The SvelteKit client (`apps/client`) is a static SPA. A board page opens one `BoardSession`
(`src/lib/board/session.svelte.ts`): a `Y.Doc`, its y-websocket room, an `Editor`, presence and
the comment threads.

```
Y.Doc ──observeDeep──▶ Scene (snapshots, z order, spatial grid, arrow bindings)
  ▲                       │ flush once per animation frame
  │ LOCAL transactions    ▼
Editor ◀── Input (pointer, wheel, pinch, keys) ──▶ tool gestures
  │
  ├──▶ scene <canvas>    background, dot grid, shapes in the viewport
  ├──▶ overlay <canvas>  selections (own and peers'), cursors, marquee, guides, drafts
  └──▶ DOM               selection handles, the text being edited, comment pins, chrome
```

| Path | Role |
|---|---|
| `src/lib/board/schema.ts` | Schema 1 types, limits and allowed keys, mirroring `docs/board-schema.md` |
| `src/lib/board/doc.ts` | `BoardDoc`: typed reads and writes over the Y.Doc, every write with the `LOCAL` origin |
| `src/lib/board/fractional.ts` | Fractional `z` keys; ties break by shape id |
| `src/lib/board/provider.ts` | y-websocket 3 wrapper: `disableBc`, stop on 1008/1009/1011, reconnect otherwise, message 100 refreshes comments |
| `src/lib/board/undo.ts` | `Y.UndoManager` tracking only `LOCAL`, so undo never reverts a peer |
| `src/lib/board/presence.ts` | Awareness `user`, `cursor`, `selection`, `viewport`, sent at most once per frame |
| `src/lib/board/assets.ts` | `uploadImage` and asset URL hooks; until the Nuage track wires them, images say "arrive soon" and no data URL enters the document |
| `src/lib/canvas/` | Camera, spatial index, hit-testing, renderer, tools, clipboard, z-order, shortcuts |
| `src/lib/canvas/ui/` | `BoardView` and its parts: toolbar, style bar, handles, text editor, minimap, comments, share dialog |

Conventions the canvas relies on:

- `rot` is radians around the box centre. Line and arrow `points` are `[x, y, ...]` relative to
  `x`, `y`; a bound end (`from`/`to`) overrides its point and follows the shape, so the stored
  point is the last known position. Deleting a bound shape freezes the arrow's ends first.
- Frames sit at the bottom of the z order when created; a shape whose centre lands in a frame
  gets `parent`, and moving a frame moves its children.
- Remote updates never paint directly: the scene queues changed ids and applies them in the next
  animation frame. The renderer culls with the spatial grid and draws a shape smaller than two
  screen pixels as one rectangle.
- Keyboard shortcuts come from `src/lib/shortcuts.ts`, the list settings shows too.
- Viewers and commenters get the select and hand tools (plus comment for commenters); the room
  refuses their document updates anyway.

`apps/client/e2e/canvas.e2e.ts` drives two members and a guest against the real API;
`CANVAS_E2E_FAKE=1` runs it against `e2e/fake.ts`, an in-memory y-websocket relay for client-only
work. `bun e2e/perf.ts 5000` seeds 5,000 shapes into that relay and reports pan and zoom frame
and paint times.

Back to the [documentation index](README.md).
