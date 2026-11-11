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
