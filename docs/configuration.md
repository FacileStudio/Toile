# Configuration

Every environment variable the API reads, from `apps/api/src/env.rs`, tronc's `env::Core` and
porte's configuration. A missing required variable, or an inconsistent pair, exits 1 at boot.

| Variable | Required | Default | What it does |
|---|---|---|---|
| `DATABASE_URL` | yes | compose: built from `TOILE_DB_PASSWORD` | Postgres connection string |
| `TOILE_DB_PASSWORD` | compose | `toile-internal-db` | Password of the compose `toile-db` and of the default `DATABASE_URL`. Set it before the first boot: Postgres keeps the first one |
| `PORT` | no | `8080` (tronc) | Listen port. The image sets `4000` |
| `APP_ENV` | no | `development` | `development`, `staging` or `production` |
| `LOG_LEVEL` | no | `info` | `debug`, `info`, `warn`, `error` |
| `TRUSTED_PROXIES` | prod | none | `private,cloudflare` behind Traefik and Cloudflare |
| `CORS_ALLOWED_ORIGINS` | no | none | Empty in production: the client is same-origin |
| `CLIENT_DIR` | no | `./client` | The built SPA. The image sets `/client` |
| `JOURNAL_URL`, `JOURNAL_TOKEN` | no | none | Ship logs to Journal when both are set |
| `OIDC_ISSUER` | prod | none | `https://sso.facile.studio`. Turns sign-in, directory sync and deletions on. Production refuses to boot without it |
| `OIDC_CLIENT_ID`, `OIDC_CLIENT_SECRET` | with issuer | none | The Registre client |
| `OIDC_REDIRECT_URL` | with issuer | none | `https://toile.facile.studio/api/auth/oidc/callback` |
| `OIDC_SUCCESS_URL` | with issuer | none | `https://toile.facile.studio/` |
| `OIDC_CLI_AUDIENCE` | no | none | Mounts `POST /api/auth/oidc/device/exchange` for that CLI client |
| `PORTE_TOKEN_KEY` | prod | none | Seals stored OIDC tokens. Production refuses to boot without it |
| `MCP_RESOURCE` | no | none | `https://toile.facile.studio/mcp`. Read and checked now; `/mcp` itself ships with track T5 |
| `MCP_STATE_KEY` | with MCP | none | HMAC key of the MCP confirmation flow; boot fails without it when `MCP_RESOURCE` is set |
| `TEST_DATABASE_URL` | tests | none | Postgres the integration tests create their schemas in |
| `APP_VERSION` | build | `dev` | Read at compile time; the Dockerfile stamps it from `apps/client/package.json` |

`SSO_ONLY` is always true: the local password routes are never registered.

## Fixed limits

| Limit | Value | Where |
|---|---|---|
| Board name | 200 characters, trimmed; blank becomes `Sans titre` | `modules/boards/service.rs` |
| Request JSON body | 1 MiB | `tronc::json` |
| Board document | see [board-schema.md](board-schema.md) | enforced once rooms ship |

Back to the [documentation index](README.md).
