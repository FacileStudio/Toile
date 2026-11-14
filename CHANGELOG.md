# Changelog

All notable changes to Toile. Versions follow SemVer.

## [Unreleased]

## [0.12.1] - 2026-11-14

### Changed

- Client: `@facile/muse` v0.23.0 and `@facile/mise-en-scene` v0.2.2

### Removed

- Landing: the download section, the "Télécharger l'app" button and the Applications link. No release carries installers yet, so every `releases/latest/download/...` link returned 404. The hero's secondary button now scrolls to the features
- README: the installer table, for the same reason

## [0.12.0] - 2026-11-14

### Changed

- API: `porte` (porte-rs) moves from v0.6.1 to v0.7.1.
- MCP: `toile_resolve_comment` is annotated destructive: resolving hides a thread and reopening clears when it was resolved.

### Fixed

- Auth: an erased person can no longer come back. The deletion loop asks `porte_identities` about every subject it already erased (`Deletions::with_holds`) and erases a re-created account again, and the sign-in kit refuses an erased subject (`Kit::with_erased`), so a token minted before the erasure gets 401 at `/mcp` instead of a fresh account.
- MCP: `toile_list_comments` and `toile_list_boards` stop a page at the result budget (about 25,000 tokens) and resume from `next_cursor`; a thread too long for one answer lists its oldest replies and sets `replies_truncated`.

## [0.11.0] - 2026-11-14

### Changed

- Landing: shorter copy, one sentence per feature, and the suite section is gone (the footer still links each app)
- Landing: muse `DownloadSection` replaces the custom download tiles, with the same release asset URLs
- Client: `@facile/muse` v0.22.0 brings the Magnetik typeface; the body no longer overrides the muse font with a system stack
- Client: `@facile/mise-en-scene` v0.2.1; the hero demo relies on the stage's own reset and reads its cursor shadow from muse tokens

## [0.10.4] - 2026-11-13

### Added

- The client loads Vision's analytics tracker on toile.facile.studio, with `data-mask` path masking, except on share links (`/s/<token>`): a share token is a capability and a 43-character token without a digit would slip past the mask
- The CSP allows `https://vision.facile.studio` in `script-src`; the loader is hashed with the other inline scripts of the build

## [0.10.3] - 2026-11-13

### Changed

- Client: `@facile/muse` moves from v0.20.0 to v0.21.1. Board tooltips open after 300 ms, at once while moving along a toolbar and on keyboard focus, one at a time.

### Fixed

- Board: every icon-only control shows a muse tooltip, including the bottom toolbar, zoom controls, style bar, Assistant trigger, comment pins, the card play button and the intro carousel dots. The top bar no longer wraps `IconButton` in a second `Tooltip`, and no control uses the native `title` any more.

## [0.10.2] - 2026-11-13

### Changed

- Deploy: healthchecks run every 30 s with a 1 s start interval.

### Fixed

- Deploy: the swarm-only `update_config` and `restart_policy` are gone from the compose file. `restart_policy: any` overrode `restart: unless-stopped`, so the containers ran with `always`.

## [0.10.0] - 2026-11-13

### Added

- MCP: `toile_create_board`, `toile_update_board` (rename, move to a space or `personal`), `toile_trash_board`, `toile_restore_board`, `toile_list_comments` (paged, open, resolved or all) and `toile_resolve_comment`
- MCP: `toile_list_boards` pages with `limit` and `cursor`
- Embed mode: `/b/<id>?embed=1` shows only the canvas, presence and an "Ouvrir dans Toile" link; Docs (`https://docs.facile.studio`) may frame Toile

### Changed

- MCP: `toile_update_shapes` is annotated destructive and idempotent; every tool description says when to call it
- Presence uses muse `Presence` and the connection state muse `ConnectionHint` (muse v0.20.0); the "En ligne" label is gone and nothing shows while the board is live

### Fixed

- Renaming a board keeps the title box the same size and place, growing only with the text

## [0.9.0] - 2026-11-13

### Added

- The sticky tool shows a translucent post-it under the pointer, FigJam style; a click drops a 200×200 square one and a drag stays square

### Changed

- The bottom toolbar groups its tools into 8 slots: Sélection (with Main), Post-it, Texte, Formes, Connecteurs, Dessin, Insérer (Cadre, Image) and Commentaire. A slot shows its last tool and picks it in one click; a second click opens its menu. Every keyboard shortcut still picks its tool directly
- The Bulle button is a floating circle with the assistant's blob, just left of the bottom toolbar, where its composer opens
- Corner resize handles are rounded squares and connection dots are solid, both in the muse blue, and the dots sit further from the selection border
- The zoom percentage stays on one line

### Removed

- The rotate handle: shapes can no longer be rotated by hand. Shapes already rotated on a board keep their angle and still resize correctly

## [0.8.0] - 2026-11-13

### Added

- Your Bulle assistant lives in Toile: its blob and name sit in the top bar and open one floating composer above the toolbar, where you draw, summarize the board or sort selected post-its
- `GET /api/boards/{id}/ai/assistant` answers the caller's Bulle assistant name (editors; never plan-gated, "Bulle" when it cannot be read)
- `accent` (`#rrggbb`) on `POST /api/boards/{id}/ai/generate` colours flowchart decisions; the composer sends your assistant blob's colour

### Changed

- AI drawings are all coloured post-its joined by arrows: mind maps colour each branch, flowcharts colour start and end, steps and decisions
- Connectors bound to a shape route as orthogonal elbows with rounded corners, Miro style, instead of Bezier curves; stored points stay the two endpoints
- A connector bound at a shape's centre (AI and MCP arrows) now starts on the side facing the other shape instead of running over it
- The minimap draws lines, arrows and ink as strokes instead of filled blocks
- The selection outline of a connector follows its rounded corners

### Removed

- The AI dropdown menu, the generate dialog and the separate summary panel, replaced by the composer

## [0.7.0] - 2026-11-13

### Added

- Board cards in the grid show a real preview of the board: an editor's open board stores its thumbnail 3 s after the last change (`PUT /api/boards/{id}/thumbnail.png`, PNG up to 512 KiB, editors only)
- Hover and keyboard-focus tooltips on the zoom buttons, colour and width swatches, back button, board name and AI menu

### Changed

- The floating Ouvrir/Copier pill above a selected card is gone; the same actions stay in the style bar
- The board grid's placeholder sketch draws solid stickies without outlines or transparency
- Minimap shapes, its viewport frame and shapes far zoomed out draw with rounded corners
- Read-only viewers see the board name as a heading instead of a disabled button

### Fixed

- Double-clicking a sticky, shape, text or frame opens it for editing again: browsers report no click count on `pointerdown`, so the board counts clicks itself

## [0.6.2] - 2026-11-13

### Fixed

- Pasting a YouTube link, or copied cards with link previews, no longer stops the board: the server now accepts the card preview fields (image, description, favicon, site)
- YouTube cards play in place with the privacy-enhanced player (youtube-nocookie.com) instead of opening a new tab
- YouTube links with share parameters (`list=`, `t=`, `si=`), Shorts, live and embed URLs all become playable cards, with their thumbnail at once
- The stop banner says why the server closed the board (access removed, board deleted, or the refused change)
- A damaged Toile clipboard payload is refused with a message instead of being pasted as text

## [0.6.1] - 2026-11-12

### Fixed

- Explicit pointer cursor style on clickable cards, buttons, toolbars, zoom controls, and the new board grid action
- Remove duplicate external link icon button on selected cards and restore south resize handle width
- Add hover status and pointer cursor on canvas link action icons and YouTube play button
- Muted tone for animated preview stickies on board grid cards

## [0.6.0] - 2026-11-12

### Added

- Canvas element right-click context menu with smart options: open/copy link for cards, edit text for text/frames, cut, copy, duplicate, delete, layer order, and lock
- Board right-click context menu to CRUD shapes: paste at clicked world coordinates, quick shape creation (sticky note, text, rectangle, frame), and viewport controls
- Interactive action buttons and floating quick-action pill on selected link preview cards
- Dedicated style toolbar actions (open and copy link) replacing color swatches when a card is selected
- Single-click hit detection on link cards: clicking YouTube play button or footer external link button immediately opens the URL without resize handle interception
- Aligned card-size quick creation button in the board grid filling empty row slots without sitting alone on a new line
- Spacious empty space under the boards grid view for comfortable bottom scrolling

## [0.5.0] - 2026-11-12

### Added

- Rich link preview cards displaying OpenGraph image thumbnails, favicons, site domains, and descriptions
- Dedicated YouTube link detection and embedding with high-resolution video thumbnails and centered play overlay
- Pasting direct image URLs (png, jpg, webp, svg, gif) automatically creates image shapes
- Direct external URL caching in canvas image loader for instant rendering without local asset storage
- Double-clicking or clicking any card opens the target link in a new browser tab

## [0.4.1] - 2026-11-12

### Added

- Rename a board directly from the card action menu and right-click context menu in `/boards`

## [0.4.0] - 2026-11-12

### Added

- Inline editable board title in the top chrome: click to edit or rename on blur and Enter
- Three-dots action trigger menu for board cards in `/boards` using `toolIcons.dots`
- Right-click context menu for board cards with quick actions (open, duplicate, delete)

### Changed

- Wheel panning and zooming batched with `requestAnimationFrame` for stutter-free scrolling
- Cached canvas bounding rectangle eliminating layout thrashing on pointer samples and wheel events
- Hardware-accelerated pattern grid rendering replacing repeated path drawing
- Native `ctx.bezierCurveTo` for curved connectors instead of sampled polylines
- Uniform grid spatial index using 32-bit integer hash keys without string allocations
- Debounced minimap redraws reducing canvas work during fast camera moves

## [0.3.0] - 2026-11-12

### Added

- Miro-style whiteboard resizing: selection borders can now be dragged to resize along a single axis, while corners resize proportionally
- Outward connection handles: 4 circular connector points positioned outside the selection frame on each side to quickly connect shapes
- Automatic cursor rotation matching the shape's orientation for corner and edge resize handles
- Target snap radius increased to 35% with automatic cardinal anchor alignment for clean connector curves

### Fixed

- Clicking outside an active text element or sticky note immediately commits and exits input editing mode
- Sticky notes keep square aspect ratio when resized from corners

## [0.2.1] - 2026-11-12

### Fixed

- The "Nouveau tableau" button on `/boards` is as tall as the name field beside it

## [0.2.0] - 2026-11-12

### Added

- Connectors bound to a shape draw as curves that leave each shape out of the side they sit on,
  while drawing and after. Free lines and arrows stay straight
- Canvas motion: the camera glides on zoom, fit, search and minimap jumps; new shapes pop in,
  deleted ones fade out; dragged stickies, cards and filled rectangles lift with a shadow.
  Bursts over 200 shapes, board loads and reduced motion skip it
- The dock, style bar and zoom controls rise in, tool buttons and swatches pop in staggered

### Changed

- `/boards` is a card grid with a preview per board: its thumbnail, or a sketch seeded by its id
  until thumbnails are written. The actions menu hides only on devices that can hover

## [0.1.0] - 2026-11-12

### Changed

- The Toile logo is the suite catalog's mark (`solar:tablet-bold-duotone`) in the rail, the
  mobile header and the landing page. The custom `toile:logo` glyph is gone
- Favicons, touch and manifest icons, the link preview image (`og-image.png`) and the head
  metadata come from the muse v0.19.0 brand generator and match the suite catalog. `og.png`,
  `og.svg`, `icon.svg` and `scripts/icons.sh` are gone
- tronc-rs v0.4.0 (features `bulle`, `mcp`) and porte-rs v0.6.1 (features `exchange`,
  `forfait`), one `tronc` crate in the tree

### Added

- The repository, bootstrapped from Docs: the gate (`scripts/check.sh`), filet, lefthook, CI,
  the distroless image and the Dokploy compose file (one replica, `toile.facile.studio`)
- API: Registre sign-in through porte-rs, directory sync, account deletions, back-channel
  logout, `/health`, `/ready`, `/version` (with `board_schema`), the API reference at `/docs`
  and the SPA
- API: `boards` (migration `0002_boards.sql`) with `GET /api/boards`, `POST /api/boards` and
  `GET /api/boards/{id}`, authorized by one `access()`
- Client: the signed-in shell (muse `SideBar`, `UserMenu`, `SettingsLayout`), the board list at
  `/boards`, and a placeholder landing at `/` for signed-out visitors
- `docs/board-schema.md`: the normative board document contract, schema 1
- API: live boards (migration `0003_live_boards.sql`): y-websocket rooms at
  `/api/boards/{id}/ws` that validate every update against schema 1, store pending updates and
  snapshot every 200 updates or 60 s, drain on shutdown and close revoked sockets; board rename,
  duplicate, trash, restore and purge, thumbnails; view, comment and edit share links with guest
  sockets; comments with the type-100 room notice; `GET /api/board-schema`
- API: `Board.role` (`owner|editor|commenter|viewer`) and `Board.thumbnail_url` replace
  `Board.access`
- Client: the public landing at `/` (an animated faux board, features, suite apps, download
  links to the latest release assets, muse legal footer), Open Graph and canonical meta, and a
  new Toile mark with its favicons, app icons and manifest (`scripts/icons.sh` regenerates them)
- Client: settings sections for appearance, keyboard shortcuts, connected apps and plan;
  `$lib/shortcuts.ts` is the one list of board shortcuts the canvas binds
- API: the `suite` module (migration `0004_suite.sql`): one porte token `Exchanger`, signed-out
  users' exchanged tokens dropped, and the forfait plans (`ai.access`, `toile.upload_mb`) fed by
  every directory snapshot and saved in `entitlement_state`; `GET /api/me/entitlements`
- Client: a `412` from a token exchange opens a prompt to sign in to Facile again
- API: board images (migration `0005_assets.sql`): `POST /api/boards/{id}/assets` sniffs PNG,
  JPEG, GIF and WebP (SVG refused), caps the size at the plan's `toile.upload_mb` (402 with
  `key` and `limit`) and stores the file in the uploader's Nuage app folder; members read it at
  `/api/boards/{id}/assets/{asset}` and link holders at `/api/share/{token}/assets/{asset}`,
  with an ETag and a year of private cache
- API: `POST /api/cards/resolve`: live cards for Docs, Feuilles, Opus and Agenda links, read
  with the viewer's own token, and Open Graph unfurls behind an SSRF guard, cached 10 minutes
- API: `POST /api/boards/{id}/shapes/{shape}/task` turns a sticky into an Opus task and writes
  its card beside the sticky through the room
- Client: image paste, drop and picker upload through `$lib/api/assets.ts`, images load from
  the board or share route, and a pasted link's card fills its title and kind from the API
- MCP at `/mcp` (tronc-rs `mcp`, porte-rs `mcpauth`, free for every account): `toile_list_boards`,
  `toile_read_board`, `toile_add_shapes` (auto-layout, refs, frames, bound arrows),
  `toile_update_shapes`, `toile_delete_shapes` (confirmed by the person) and `toile_comment`;
  writes go through the board's room. Migration `0006_mcp_meetings.sql` adds `mcp_actions`
- API: `GET /api/mcp/actions` and its confirm and dismiss routes; client: the `/pending` page
- API: `POST /api/boards/meeting`, one board and edit link per (host, Echo room), with a
  credentialed CORS rule for `https://echo.facile.studio` on that route only
- API: the CSP allows `connect-src https://api-ardoise.facile.studio` for the plan catalog
- Native apps: a Tauri 2 desktop shell (`apps/desktop`) and a Capacitor 8 Android and iOS shell
  (`apps/client/android`, `apps/client/ios`). They sign in through porte's CLI flow (a loopback
  redirect on the desktop, a pasted code on a phone), keep the session token in the OS keychain
  or the Android Keystore, and call the API with a bearer
- API: CORS always allows the four app origins, and the board socket accepts the bearer as a
  `bearer.<token>` WebSocket subprotocol
- `.github/workflows/release.yml`: a `desktop-v*` tag builds the macOS dmg (arm64, x64), the
  Linux deb and AppImage, the Windows msi and the Android APK, and publishes them under fixed
  names
