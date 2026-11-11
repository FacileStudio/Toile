# Toile v1 plan

Checked against: Docs repo layout and gate (`~/Code/Facile/Docs`, CLAUDE.md, filet.yml, check.sh,
Dockerfile, docker-compose.yml, ci.yml), standards/migrations, conventions/facile-spaces,
conventions/agent-access-facile-apps, conventions/facile-auth-screens, syntheses/suite-mcp-plan,
syntheses/facile-forfait-billing-contract, tools/dokploy, bugs/tronc-cors-unset-breaks-webview-clients,
projects/courrier + projects/sablier (Tauri + Capacitor shells), Registre `seed.yaml`, owner rules
(muse, no borders, one user menu, AI paid and through Bulle, MCP free, Rust only where the engine is Rust).

## Goal

Ship Toile, the suite's collaborative infinite canvas, at `toile.facile.studio`, with desktop
(Tauri) and mobile (Capacitor) apps, on the same footing as Docs.

## Why (evidence)

The owner retired the personal Toile (Dokploy application and DNS record deleted 2026-11-07 and
2026-11-11, `tools/dokploy.md`) and asked for a Miro/FigJam-class suite app. No suite app draws
boards today.

## Approach

Copy Docs' shape, not its code paths blindly: Rust API on tronc-rs + porte-rs with yrs rooms held
in memory (one replica), Postgres for boards, snapshots, share links and comments, a SvelteKit +
muse client with its own canvas engine on Yjs, and one container behind Traefik. The board
document is a Y.Doc with a fixed, versioned schema that the server validates on every update (the
Docs "every write validates" invariant). Export, thumbnails and MCP read the Y.Doc server-side.
Suite calls (Nuage uploads, Bulle AI, Opus tasks, live cards) use Registre RFC 8693 token exchange,
which porte-rs lacks today: track L adds it to the libraries instead of hand-rolling it in Toile.

## Decisions (owner, 2026-11-11)

| Topic | Decision |
|---|---|
| Old repo | Rename `FacileStudio/Toile` to `Toile-legacy`, archive it, create a fresh `FacileStudio/Toile` |
| Backend | Rust like Docs (tronc-rs, porte-rs, yrs) |
| Canvas | Own Svelte engine on Yjs, `perfect-freehand` for ink |
| Storage | Server boards. Export PNG, SVG, PDF, JSON Canvas. Import JSON Canvas and Excalidraw. No migration of old boards |
| v1 | Core board, live collaboration, export and import. Workshop tools later |
| Integrations | Live suite link cards, sticky to Opus task, free MCP, paid AI via Bulle (`ai.access`), Echo meeting boards |
| Sharing | Registre spaces + view/comment/edit links; a guest types a display name once; links live until revoked |
| Uploads | Into the uploader's Nuage (folder `Toile/<board>`); guests cannot upload |
| Landing | Public muse landing at `/` for signed-out visitors; downloads from GitHub releases `latest` |
| Plans | No board cap. Plan gates only AI (`ai.access`) and upload size (`toile.upload_mb`) |

## Board document contract (schema 1)

Written first as `docs/board-schema.md`, mirrored by `apps/client/src/lib/board/schema.ts` and
`apps/api/src/modules/board/schema.rs`. Every track codes against it.

- `meta`: Y.Map `{ schema: 1 }`.
- `shapes`: Y.Map<id, Y.Map>. Common keys: `type`, `x`, `y`, `w`, `h`, `rot`, `z` (fractional
  index string), `parent` (frame id or absent), `style` (plain object: `fill`, `stroke`, `width`,
  `font`, `size`, `align`), `locked`.
- Types: `sticky`, `text`, `rect`, `ellipse`, `diamond` (text in `text`: Y.Text), `line`,
  `arrow` (`points`, `from`/`to`: `{ shape, anchor: [0..1, 0..1] }` bindings, `head`), `ink`
  (`points`: flat number array `[x, y, pressure, ...]`, relative to `x`,`y`), `image` (`asset`:
  Toile asset id), `card` (`url`, cached `title`, `kind`), `frame` (`name`).
- Limits checked by the server: 20,000 shapes, 10,000 ink points per stroke, 10,000 chars per
  text, 1 MiB per update, numbers finite and within ±1e7, unknown types or keys refused.
- Presence (awareness, never stored): `user {name, color, picture?, guest}`, `cursor`,
  `selection`, `viewport` (for follow mode).
- Comments are NOT in the Y.Doc: Postgres rows anchored to `{shape?, x, y}`, broadcast to the room
  as a custom message so open clients refresh.

## Parallel tracks

```
T0 bootstrap ──┬── T2 rooms/boards API ──┬── T5 integrations ── T9 hardening + ship
               ├── T3 canvas client ─────┤
               ├── T4 export/import ─────┤
               ├── T6 native shells      │
               ├── T7 landing + settings │
L libraries ───┴─────────────────────────┘
T8 cross-repo (Registre, Nuage, muse catalog, Echo PR) after T0
```

Every agent works in its own git worktree under `/srv/scratch/cache/toile/<track>` on branch
`feat/<track>`, with `CARGO_TARGET_DIR=/srv/scratch/cache/cargo-target-toile` (shared; cargo's lock
serialises builds) and `CARGO_PROFILE_DEV_DEBUG=0`. `/` has 11 GB free and is the Dokploy host's
disk: never build Rust under `~/Code`. A track merges to `main` through a PR only when
`sh scripts/check.sh` and `filet check .` pass; the lead merges in dependency order and rebases
the rest.

### L. Library gaps (parallel with T0)

1. `porte-rs/src/exchange/` (feature `exchange`): RFC 8693 client mirroring Go `porte/exchange`
   v0.15.1: subject token = the user's stored identity access token (refreshed with the stored
   refresh token, `PORTE_TOKEN_KEY`), audience + scopes in, 5-minute JWT out, small cache keyed
   on (user, audience, scopes). Tests against a fake token endpoint. Tag a minor release.
2. `tronc-rs/src/bulle/` (feature `bulle`): the Rust twin of Go `tronc/bulle` v0.18.0 (client for
   Bulle's completion API, given an exchanged token). Tag a minor release.
3. Check `porte-rs` `directory` exposes entitlements (`snapshot.rs` mentions them); if no
   `forfait::check(person, key)` helper exists, add one beside it mirroring Go `porte/forfait`.

Exit: tags pushed, `cargo test` green in each repo, filet clean, CHANGELOG entries.

### T0. Bootstrap (serial, blocks T2-T7)

1. GitHub: `gh repo rename Toile-legacy -R FacileStudio/Toile`, `gh repo archive`, then
   `gh repo create FacileStudio/Toile --public`. Local: move `~/Code/Facile/Toile` to
   `Toile-legacy`, remove `Toile-wt-deps` (old worktree, 4.5 GB), clone the new repo to
   `~/Code/Facile/Toile`.
2. Scaffold from Docs: `mise.toml`, `filet.yml` (same limits, `requiredDirs` adds `apps/desktop`),
   `lefthook.yml`, `.github/workflows/ci.yml`, `scripts/{check,e2e,postgres-up,postgres-down,
   version,verify-deploy}.sh` (DB port 5657), `Dockerfile`, `docker-compose.yml` (toile-db,
   toile-api, host `toile.facile.studio`, `replicas: 1`, stop-first), `LICENSE` (AGPL-3.0-only),
   `README.md`, `CLAUDE.md`, `CHANGELOG.md`, `docs/{README,architecture,api,configuration,
   deployment,development,board-schema}.md`.
3. `apps/api`: `main.rs`, `server.rs`, `router.rs`, `state.rs`, `env.rs`, `security.rs`,
   `version.rs`, `apiref.rs`, `auth/*` and `modules/spaces` adapted from Docs; migrations
   `0001_porte.sql` (porte schema verbatim), `0002_toile.sql` (empty boards table to start)
   [migrations: append-only `NNNN_name.sql`]. Health, `/version`, SPA, OIDC sign-in.
4. `apps/client`: SvelteKit 2, Svelte 5 runes, Tailwind 4, `@facile/muse` latest tag,
   `@facile/client`; signed-in shell with muse SideBar, UserMenu, settings layout; board list
   placeholder [muse: tokens only, no borders, mobile first].
5. Deploy skeleton (see "Deploy" below) so every later merge to `main` autodeploys.

Exit: `check.sh` + `filet check .` green in CI, `https://toile.facile.studio/health` 200, sign-in
through Registre works end to end.

### T2. Boards, rooms, sharing, comments (API)

1. `migrations/0003_boards.sql`: `boards` (id, owner user, space?, name, created/updated,
   deleted_at, thumbnail), `board_state` (snapshot bytea, version), `board_updates` (pending),
   `share_links` (token hash, board, role view|comment|edit, created_by, revoked_at),
   `comments` (id, board, anchor jsonb, body, author user or guest name, resolved_at, parent).
2. `modules/boards/{router,handlers,service,access,trash}.rs`: CRUD, list by scope, duplicate,
   trash/restore/purge. One `access()` returns the role or `None` (404), like Docs.
3. `modules/board/schema.rs`: the schema 1 validator over a yrs doc.
4. `modules/rooms/*`: Docs' room code adapted: hub, room task, sync, awareness, persist
   (snapshot every 200 updates or 60 s), drain on shutdown, validate every update and revert +
   close 1008 on failure, viewers and commenters may send awareness only, revoked links close
   sockets.
5. `modules/share/*`: create/list/revoke links; `GET /api/share/{token}` resolves board + role;
   guest WebSocket with the share token; guest name 1-40 chars, stored on comments.
6. `modules/comments/*`: threads, resolve, delete own; broadcast `comments-changed` to the room;
   rate limit guests.
7. Tests in `tests/` mirroring Docs: authorization, rooms (two clients converge), durability
   (restart keeps state), share (each role), limits (oversized/invalid updates refused).

Exit: tests green with Postgres, a 50-client room soak (`tests/soak.rs`, ignored by default) holds
under 100 ms fan-out.

### T3. Canvas client

1. `src/lib/board/{schema,doc,provider}.ts`: typed accessors over the Y.Doc, y-websocket provider
   wrapper (stop on 1008/1009/1011, `disableBc: true`, like Docs), `Y.UndoManager` limited to the
   local origin.
2. `src/lib/canvas/`: camera (pan, zoom to cursor, pinch), spatial index (grid or R-tree) for
   hit-testing and culling, renderer (one `<canvas>` for shapes and ink, DOM overlay only for the
   text being edited and for selection handles), rAF-batched remote updates.
3. Tools: select (marquee, multi-select, move, resize, rotate, snap), sticky, text, shapes,
   connector arrows bound to shapes, pen + eraser (`perfect-freehand`), image (paste, drop, file
   picker), frame, hand. Keyboard shortcuts with a `?` cheatsheet, z-order, lock, duplicate,
   copy/paste between boards, minimap, zoom controls, `⌘K` board search.
4. Presence: live cursors with names, selection outlines, follow a user, avatars stack.
5. Comments UI: pins on the board, thread side panel.
6. Routes: `/boards` list, `/b/[id]`, `/s/[token]` guest (name prompt, localStorage), share
   dialog.
7. Tests: vitest for camera math, hit-testing, fractional index, schema accessors; Playwright e2e:
   two browsers draw and see each other, guest view link cannot edit.

Exit: 5,000-shape board pans at 60 fps on a mid laptop (measured with a seeded board), e2e green.

### T4. Export and import (API)

1. `modules/export/svg.rs`: one SVG writer for schema 1 (used for SVG export, thumbnails, MCP
   previews), `png.rs` (resvg), `pdf.rs` (svg2pdf), `jsoncanvas.rs`.
2. `modules/import/{jsoncanvas,excalidraw}.rs`: map to schema 1, validate, create a board.
3. Routes: `GET /api/boards/{id}/export.{svg,png,pdf,canvas}` (optional `frame=`),
   `POST /api/boards/import`. Thumbnails refreshed on persist, debounced.

Exit: round-trip test JSON Canvas -> board -> JSON Canvas keeps nodes and edges; snapshot tests of
the SVG writer.

### T5. Suite integrations (after L and T2)

1. Uploads: `modules/assets/*`: multipart upload, size gate `toile.upload_mb` (default 10), sniff
   type, store in the uploader's Nuage `Toile/<board>` through an exchanged token, create a Nuage
   share for read access, keep `assets` row (board, nuage file id, share token); `GET
   /api/boards/{id}/assets/{asset}` proxies with board access, so guests see images.
2. Live cards: `modules/cards/*`: recognise Nuage, Docs, Feuilles, Opus, Agenda URLs, fetch title
   and status through exchanged tokens for the viewing user, fall back to an Open Graph unfurl for
   other URLs (SSRF guard: https only, public IPs only, 1 MB, 5 s).
3. Sticky to Opus task: `POST /api/boards/{id}/shapes/{shape}/task` creates the task, stores the
   link on the shape as a card.
4. AI via Bulle: `modules/ai/*`: summarise board, cluster stickies into frames, generate a
   mindmap or diagram from a prompt; gate `ai.access` returns 402 with the muse plan-limit body;
   writes go through the room so editors see them live.
5. MCP: `src/mcp/*` on tronc-rs `mcp`: `toile_list_boards`, `toile_read_board` (shapes as text +
   SVG preview link), `toile_add_shapes`, `toile_update_shapes`, `toile_comment`; writes through
   the room; free for every account.
6. Echo: `POST /api/boards/meeting` returns a board for an Echo room id (idempotent per room) and
   an edit share link for participants.

Exit: integration tests with fake Nuage/Opus/Bulle servers; `mcp-live-smoke` flow passes against
production with `MCP_TOOL=toile_list_boards`.

### T6. Native shells (after T0)

1. `apps/desktop`: Tauri 2 crate copied from Courrier's shell (own workspace, version from
   `apps/client/package.json`, bundle id `studio.facile.toile`, updater pointed at GitHub
   releases latest).
2. Capacitor 8 in `apps/client` (`capacitor.config.ts`, appId `studio.facile.toile`), Android
   project committed, iOS project committed (built on a Mac later), `$lib/platform.ts` +
   `$lib/native.ts` like Sablier: bearer in secure storage, no CSRF header when native,
   API origin `https://toile.facile.studio`.
3. Native sign-in through porte-rs `oidc` device/bearer flow; tronc CORS lists `https://localhost`,
   `capacitor://localhost`, `tauri://localhost`, `http://tauri.localhost`.
4. `.github/workflows/release.yml`: `desktop-v*` tag builds macOS dmg (arm64, x64), Linux deb +
   AppImage, Windows msi, signed Android APK with fixed asset names.
5. Touch: pinch zoom, two-finger pan, Apple Pencil/stylus pressure, palm rejection in pen mode.

Exit: `bun run tauri build` produces a working Linux build that signs in; Android debug APK signs
in on an emulator; release workflow green on a test tag.

### T7. Landing and settings (after T0)

1. `/` for signed-out visitors: muse landing (hero with a live mini canvas, features, download
   buttons linking to `releases/latest/download/<asset>`, legal links via muse `legal.ts`).
2. Settings with the shared muse layout: profile, appearance, shortcuts, connected apps, plan.
3. `static/` icons, manifest, favicon from one SVG.

Exit: Lighthouse accessibility ≥ 95 on `/`, no borders anywhere.

### T8. Cross-repo changes (after T0; each a PR in its own repo)

1. Registre `seed.yaml`: `Toile` client (redirect `https://toile.facile.studio/api/auth/oidc/callback`,
   sync, deletions, back-channel logout, MCP resource `https://toile.facile.studio/mcp`,
   scopes `toile.read toile.write`, token exchange to Nuage, Opus, Docs, Feuilles, Agenda, Bulle),
   Bulle's grant list gains Toile. Secret in Registre's Dokploy env `REGISTRE_SEED_TOILE_SECRET`.
2. Nuage: `NUAGE_APPS` gains `toile:<client_id>:Toile` (env change through `dokploy-env-safe`).
3. Opus, Docs, Feuilles, Agenda: confirm each accepts Toile as `act.sub`; add it where an
   allow-list exists.
4. muse: add Toile to the app catalog/launcher if one lists apps; tag a release.
5. Echo: a "Whiteboard" button calling Toile's meeting endpoint. Opened as a PR and held for the
   owner: Echo keeps Noah's design.
6. Suite landing (`FacileStudio/Suite`): add Toile to the app list.

### T9. Production hardening and ship

1. `/facile-review` over the whole repo; fix every finding at error level.
2. Security: CSP (report-only first, like Registre), rate limits on share and guest routes,
   upload sniffing, SSRF guard test, authorization test for every route (the Docs pattern).
3. Observability: Journal (`JOURNAL_URL`, `JOURNAL_TOKEN`), a Sonde monitor on `/health`, an
   Antenne alert on failed deploys if the suite has one.
4. Backups: add `toile_db_data` to whatever backup job exists; if none (ops-backups says none is
   automated), record that in the deployment doc.
5. Release `v1.0.0` with `/ship`, then `desktop-v1.0.0`.
6. Verify: `deploy-verify` flow, `mcp-live-smoke`, two real browsers on production.

## Deploy

1. DNS: `A toile.facile.studio 65.108.2.107`, proxied, via the Cloudflare API with
   `tiroir get CF_API_TOKEN` (record deleted 2026-11-11; backup JSON in `~/backups/`).
2. Dokploy: compose `Toile` in project Facile / production, GitHub `FacileStudio/Toile` branch
   `main`, `composePath ./docker-compose.yml`, autoDeploy on. Env: `OIDC_*`, `PORTE_TOKEN_KEY`,
   `MCP_RESOURCE`, `MCP_STATE_KEY`, `JOURNAL_*`, Postgres password; set with `dokploy-env-safe`.
3. Before the first build, check `df -h /` and the Docker build cache; a Rust image build needs
   several GB on `/`.
4. Wait with `dokploy-deploy-wait`, then `deploy-verify` with `VERIFY_URLS=https://toile.facile.studio/health`.

## Exit criteria (v1 done)

- CI green on `main`: fmt, clippy `-D warnings`, cargo tests with Postgres, client check + test +
  build, Playwright e2e, filet.
- `https://toile.facile.studio` serves the landing, signs in through Registre, two users edit one
  board live, a guest link works for each role, exports download, MCP smoke passes.
- Desktop and Android builds attached to the `desktop-v1.0.0` release.
- Docs folder complete per standards/docs.

## Risks

- Disk: `/` at 99%. Rust targets go to `/srv/scratch`; the Docker build may still need a builder
  prune on the host (ask the owner).
- porte-rs token exchange is new library code on the auth path: it gets its own review.
- Uploads into Nuage depend on Nuage shares for guest reads; if Nuage refuses app-created shares,
  fall back to Toile proxying with the uploader's exchanged token at read time.
- One replica: a deploy disconnects every board for a few seconds, like Docs.
- Two renderers (canvas client, SVG server) can drift; the SVG snapshot tests cover the schema.
- Dokploy builds one compose at a time; deploys can wait 2 to 25 minutes behind other apps.
- iOS cannot be built on this Linux host; the iOS project ships unbuilt.

## Skip (YAGNI)

Timer, voting, stamps, templates, offline-first local boards, end-to-end encryption, pub/sub for
multiple replicas, version history UI (snapshots exist, no browsing UI), presentation mode, old
Toile migration, Windows code signing.
