# Changelog

All notable changes to Toile. Versions follow SemVer.

## [Unreleased]

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
- API: the CSP allows `connect-src https://api-ardoise.facile.studio` for the plan catalog
