# Changelog

All notable changes to Toile. Versions follow SemVer.

## [Unreleased]

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
- API: `POST /api/boards/meeting`, one board and edit link per (host, Echo room)
- API: the CSP allows `connect-src https://api-ardoise.facile.studio` for the plan catalog
