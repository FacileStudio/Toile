# Toile

[![ci](https://github.com/FacileStudio/Toile/actions/workflows/ci.yml/badge.svg)](https://github.com/FacileStudio/Toile/actions/workflows/ci.yml)

The collaborative infinite canvas of the Facile suite: sticky notes, shapes, arrows and ink on
one board, drawn together in real time. Toile runs at `toile.facile.studio`.

This is the v1 bootstrap. Today the app signs in through Registre, lists and creates boards per
scope (personal or a Registre space) and follows the directory. The canvas, live rooms, sharing,
export and the suite integrations land next: [docs/plan.md](docs/plan.md).

## Stack

| Layer | Tech |
|---|---|
| API | Rust 1.98, axum 0.8, sqlx 0.9, PostgreSQL 16, tronc-rs 0.3, porte-rs 0.5 |
| Client | SvelteKit 2, Svelte 5 (runes), Tailwind CSS 4, `@facile/muse`, `@facile/client` |
| Storage | PostgreSQL 16 |
| Deploy | Docker Compose, one container and one replica behind Traefik |

## Quick start

```sh
docker compose up -d --build
```

Compose reads the variables of [docs/configuration.md](docs/configuration.md) from a `.env`
beside it.

### Local development

```sh
mise trust && mise install
mise run db-up
mise run dev
```

The API listens on <http://localhost:4000>; run the client from `apps/client` with `bun run
dev`. The API reference is at <http://localhost:4000/docs>.

## Downloads

No installers are published yet: every [release](https://github.com/FacileStudio/Toile/releases)
so far ships without assets. Use Toile in the browser at https://toile.facile.studio. Once the
release workflow uploads its builds, they land at fixed names under `releases/latest/download/`.

## Configuration

| Variable | What it does |
|---|---|
| `DATABASE_URL` | Postgres connection string |
| `TOILE_DB_PASSWORD` | The compose Postgres password, used by the default `DATABASE_URL` |
| `OIDC_ISSUER`, `OIDC_CLIENT_ID`, `OIDC_CLIENT_SECRET` | Registre sign-in and the directory |
| `OIDC_REDIRECT_URL`, `OIDC_SUCCESS_URL` | The callback and landing URLs |
| `PORTE_TOKEN_KEY` | Seals stored OIDC tokens; required in production |
| `MCP_RESOURCE`, `MCP_STATE_KEY` | Read now, used once `/mcp` ships |

Full reference: [docs/configuration.md](docs/configuration.md).

## Structure

```
apps/
  api/       Rust API: auth/ (porte), modules/ (boards, spaces), migrations/, tests/
  client/    SvelteKit client, with the Capacitor android/ and ios/ projects
  desktop/   Tauri 2 desktop shell
docs/        Architecture, board schema, configuration, development, deployment, API, plan
scripts/     check.sh, e2e.sh, version.sh, postgres-up.sh, postgres-down.sh, verify-deploy.sh
```

## Documentation

| Doc | What's in it |
|---|---|
| [Architecture](docs/architecture.md) | Request flow, access, data model |
| [Board schema](docs/board-schema.md) | The normative Y.Doc contract every track codes against |
| [Configuration](docs/configuration.md) | Every environment variable and default |
| [Development](docs/development.md) | Local setup, tests, the quality gate |
| [Deployment](docs/deployment.md) | Docker Compose, Dokploy, Traefik routing |
| [API](docs/api.md) | HTTP endpoints |
| [Plan](docs/plan.md) | The v1 build plan |

## License

Toile is licensed under the [GNU Affero General Public License v3.0](LICENSE) (`AGPL-3.0-only`).
A commercial licence is available from Facile Studio for use that cannot meet the AGPL's terms.

---

Part of the [Facile Suite](https://facile.studio): self-hosted tools for creative studios and
freelancers. One login, zero cloud dependency.
