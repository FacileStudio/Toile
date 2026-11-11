# Toile

The collaborative infinite canvas of the Facile suite at `toile.facile.studio`. Rust API in
`apps/api`, SvelteKit client in `apps/client`, one container. The v1 plan is `docs/plan.md`; the
board document contract is `docs/board-schema.md` (normative).

## Stack

- **API** (`apps/api`): Rust 1.98 (edition 2024), axum 0.8, sqlx 0.9 on Postgres 16
- **Libraries**: `tronc` (tronc-rs v0.4.0), `porte` (porte-rs v0.6.1), from their public
  `github.com/FacileStudio` repos by tag (bump: `docs/deployment.md`). Never copy their code
  here; report a gap instead. Toile does not depend on encre
- **Client** (`apps/client`): SvelteKit 2, Svelte 5 runes, Tailwind 4, `@facile/muse`,
  `@facile/client`

## Commands

```sh
mise run db-up              # Postgres 16 on 127.0.0.1:5657 (container toile_dev_postgres)
sh scripts/check.sh         # the gate: fmt, clippy -D warnings, tests, the client, then e2e
mise run e2e                # Playwright + headless Chromium against the real API and Postgres
filet check .               # after the gate, before every commit
mise run dev                # the API on :4000 against the local database
cd apps/api && cargo run -- migrate status
```

On the shared build host, export `CARGO_TARGET_DIR=/srv/scratch/cache/cargo-target-toile` and
`CARGO_PROFILE_DEV_DEBUG=0` first: never let a `target/` grow under `~/Code`.

The database tests read `TEST_DATABASE_URL` (check.sh defaults it to the local container) and
print a skip banner without it: read the output before trusting a green run.

## Invariants

- **One replica.** Rooms will live in memory; two replicas split a room. `docker-compose.yml`
  pins `replicas: 1` and `stop-first`. Do not scale it until rooms have a pub/sub layer.
- **Every Yjs update validates.** A room checks each update against board schema 1
  (`docs/board-schema.md`); a failing update is reverted to the exact prior state and its sender
  closed with 1008. REST, MCP and AI writes validate the same way.
- **Writes go through the room.** REST, MCP, AI and import writes to an open board run inside its
  room, so open clients see them live.
- **One `access()`.** Every board query is authorized by `modules/boards/access.rs`. `None` is
  always 404; lists filter by scope in the same SQL statement.
- **Migrations** are append-only `NNNN_name.sql`; `0001_porte.sql` is `porte::pg::SCHEMA`
  verbatim. Never edit a shipped migration.
- No inline comments, no comments in function bodies; doc comments on public items.
- UI: muse tokens and components only, no borders anywhere, one user menu (muse `UserMenu`
  through `SideBar`), one settings layout (muse `SettingsLayout`).
