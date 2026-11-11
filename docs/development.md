# Development

How to run the API locally, test it, and pass the gate before a commit.

## Prerequisites

- Rust 1.98.1 (`apps/api/rust-toolchain.toml`) with `rustfmt` and `clippy`
- Docker, for Postgres 16, and `pg_isready` (psql 16 or later)
- bun 1.4 for the client, `filet` 0.19 or later, `mise` and `lefthook`
- Network access to GitHub: `tronc-rs` (`v0.3.0`) and `porte-rs` (`v0.5.2`) come from their
  public repos by tag

On the shared build host, export `CARGO_TARGET_DIR=/srv/scratch/cache/cargo-target-toile` and
`CARGO_PROFILE_DEV_DEBUG=0` before any cargo command. `apps/client/e2e/stack.ts` reads
`CARGO_TARGET_DIR` to find the binaries.

## Setup

```sh
mise trust && mise install
mise run db-up
```

`db-up` starts `postgres:16-alpine` as `toile_dev_postgres` on `127.0.0.1:5657` (password
`postgres`) with `fsync` off. `mise run db-down` stops it.

## Run

```sh
mise run dev
```

The API listens on `:4000` with no Registre: sign-in routes answer `oidc_enabled: false`. The
client's `bun run dev` proxies `/api` to it.

## Tests

```sh
export TEST_DATABASE_URL='postgres://postgres:postgres@127.0.0.1:5657/postgres?sslmode=disable'
cd apps/api && cargo test --locked
```

Unit tests sit beside the code. Integration tests in `apps/api/tests/` build the real router
against Postgres through `tronc::testdb`, one schema per test prefix:

| File | Covers |
|---|---|
| `authorization.rs` | Every board route for each role, the member of another space and anonymous; personal boards |
| `lifecycle.rs` | Create and list per scope, ordering, deleted boards, bad bodies, probes and the API reference drift test |
| `directory.rs` | porte's spaces conformance suite, snapshot apply, retired spaces, the empty-snapshot refusal, GDPR erase |

Without `TEST_DATABASE_URL` the database tests print a skip banner and pass: read the output.

The client's unit tests run with `cd apps/client && bun run test` (vitest, jsdom).

## End to end

```sh
mise run e2e
```

`scripts/e2e.sh` builds the debug `api` binary and the test-only `e2e_seed` example, builds the
client, then runs `apps/client/e2e/toile.e2e.ts` with `bun test` and `playwright-core` driving
headless Chromium. `e2e/stack.ts` recreates the `toile_e2e` database on the local Postgres
(`E2E_ADMIN_URL`, default port 5657), starts the API on a free port with
`CLIENT_DIR=apps/client/build`, then runs `e2e_seed`, which writes alice (owner), bob (member)
and carol (outsider) and mints each a porte session. The run proves the signed-out landing and
that a member creates a board that survives a reload.

It needs Playwright's Chromium headless shell under `~/.cache/ms-playwright`:
`cd apps/client && bunx playwright-core install chromium-headless-shell`. `check.sh` skips it
with a message when Postgres or that browser is missing, or when `SKIP_E2E` is set.

## The gate

```sh
sh scripts/check.sh
filet check .
```

`check.sh` runs `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings` and
`cargo test --locked` in `apps/api`, then `bun run check`, `bun run test` and `bun run build` in
`apps/client`, then the end-to-end run when it can. `lefthook` runs it on pre-push. Never raise a
filet limit to pass.

Back to the [documentation index](README.md).
