# Deployment

How Toile ships: one image, one container, one Traefik router, one hostname, one replica.

## Topology

```
toile.facile.studio ──▶ Traefik (web → websecure) ──▶ toile-api:4000 ──▶ toile-db:5432
                                                                          volume toile_db_data
```

`/api/*` is the API, `/docs` the API reference, everything else the SPA. There is no
`PathPrefix` and no `stripprefix`.

## One replica

Board rooms will be held in memory: two replicas would each hold a different copy of an open
board. `docker-compose.yml` pins `deploy.replicas: 1` and `update_config.order: stop-first`, so a
deploy never runs two containers side by side. A deploy disconnects every open board for a few
seconds. This holds until rooms get a pub/sub layer.

## Build

```sh
docker build -t toile .
```

The root `Dockerfile` builds the client with bun, builds the API with `cargo build --release
--locked` (the version comes from `apps/client/package.json` through `scripts/version.sh`, and
the build fails when it cannot be read), and copies both into
`gcr.io/distroless/cc-debian12:nonroot`, pinned by digest. The API runs as `nonroot` (uid 65532)
and writes nothing to disk. The entrypoint is `/api`, so `docker run toile migrate status` works.

A Rust image build needs several GB on `/`. Check `df -h /` and the Docker build cache before the
first deploy.

## Dokploy

Compose `Toile` in project Facile, environment production, from `FacileStudio/Toile` branch
`main`, `composePath ./docker-compose.yml`, autodeploy on. The compose file forwards every
variable explicitly: set `TOILE_DB_PASSWORD`, `OIDC_*`, `PORTE_TOKEN_KEY`, `MCP_RESOURCE`,
`MCP_STATE_KEY` and `JOURNAL_*` in the Dokploy environment with the `dokploy-env-safe` flow.
Set `TOILE_DB_PASSWORD` before the first deploy: Postgres keeps the password of its first boot
in `toile_db_data`.

## Migrations

Migrations run at boot, behind sqlx's lock. `0001_porte.sql` is porte's schema verbatim,
`0002_boards.sql` the directory tables and `boards`. They are append-only.

## Health

`/health` and `/ready` (also under `/api`). The compose healthcheck re-execs the binary:
`["CMD", "/api", "healthcheck"]`. After a deploy run `sh scripts/verify-deploy.sh
toile.facile.studio` or the `deploy-verify` flow.

## Bumping a library

`tronc-rs` and `porte-rs` come from their public GitHub repos, pinned by tag. To move one, change
its `tag` in `apps/api/Cargo.toml` (tronc appears twice, and porte pins its own tronc: keep them
on the same tag so `cargo tree` shows one copy), then run `cargo update -p <crate>` and commit
`Cargo.lock`. `@facile/muse` and `@facile/client` are `github:FacileStudio/<repo>#<tag>` in
`apps/client/package.json`, followed by `bun install`. Run `sh scripts/check.sh` before the
commit.

## Registering in the suite

Registre `seed.yaml` gets a `Toile` client: redirect
`https://toile.facile.studio/api/auth/oidc/callback`, sync `/api/registre/sync`, deletions
`/api/registre/deletions`, back-channel logout `/api/auth/backchannel-logout`, MCP resource
`https://toile.facile.studio/mcp` with scopes `toile.read toile.write`. Details in plan track T8.

Back to the [documentation index](README.md).
