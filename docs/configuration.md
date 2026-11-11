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
| `CORS_ALLOWED_ORIGINS` | no | none | Extra origins. The four app origins (`https://localhost`, `capacitor://localhost`, `tauri://localhost`, `http://tauri.localhost`) are always added |
| `CLIENT_DIR` | no | `./client` | The built SPA. The image sets `/client` |
| `JOURNAL_URL`, `JOURNAL_TOKEN` | no | none | Ship logs to Journal when both are set |
| `OIDC_ISSUER` | prod | none | `https://sso.facile.studio`. Turns sign-in, directory sync and deletions on. Production refuses to boot without it |
| `OIDC_CLIENT_ID`, `OIDC_CLIENT_SECRET` | with issuer | none | The Registre client |
| `OIDC_REDIRECT_URL` | with issuer | none | `https://toile.facile.studio/api/auth/oidc/callback` |
| `OIDC_SUCCESS_URL` | with issuer | none | `https://toile.facile.studio/` |
| `OIDC_CLI_AUDIENCE` | no | none | Mounts `POST /api/auth/oidc/device/exchange` for that CLI client |
| `PORTE_TOKEN_KEY` | prod | none | Seals stored OIDC tokens. Production refuses to boot without it |
| `MCP_RESOURCE` | no | none | `https://toile.facile.studio/mcp`. Turns on `/mcp` |
| `MCP_STATE_KEY` | with MCP | none | HMAC key of the MCP confirmation flow; boot fails without it when `MCP_RESOURCE` is set |
| `NUAGE_URL`, `OPUS_URL`, `DOCS_URL`, `FEUILLES_URL`, `AGENDA_URL` | no | `https://<app>.facile.studio` | Public origin of each suite app Toile calls; links to it become live cards |
| `NUAGE_CLIENT_ID` | for uploads | none | Nuage's Registre client id, the audience of upload tokens. Without it uploads answer 503 |
| `OPUS_CLIENT_ID`, `DOCS_CLIENT_ID`, `FEUILLES_CLIENT_ID`, `AGENDA_CLIENT_ID` | no | none | Each app's Registre client id. Without one its links stay host-only cards, and without Opus's, sticky-to-task answers 503 |
| `TEST_DATABASE_URL` | tests | none | Postgres the integration tests create their schemas in |
| `APP_VERSION` | build | `dev` | Read at compile time; the Dockerfile stamps it from `apps/client/package.json` |

`SSO_ONLY` is always true: the local password routes are never registered.

Token exchange to other suite apps reuses `OIDC_ISSUER`, `OIDC_CLIENT_ID` and
`OIDC_CLIENT_SECRET` and the refresh token stored at sign-in (sealed by `PORTE_TOKEN_KEY`). It
needs Registre's seed to grant Toile `offline_access` and a `token_exchange` entry per audience.
Each `<APP>_CLIENT_ID` must equal that app's `OIDC_MACHINE_AUDIENCE`, and Nuage must list Toile in
`NUAGE_APPS` so its tokens reach `/api/app/*`.

## Plans

The directory snapshot carries the plan values; the last one applied is saved in the
`entitlement_state` row and restored at boot, so a restart never fails open again. Until the
first snapshot with entitlements, every feature is on and unlimited.

| Feature | Kind | Free plan | What it gates |
|---|---|---|---|
| `ai.access` | bool | `false` | AI features through Bulle |
| `toile.upload_mb` | limit | `10` | The largest image one upload may be, in MB |

## Fixed limits

| Limit | Value | Where |
|---|---|---|
| Board name | 200 characters, trimmed; blank becomes `Sans titre` | `modules/boards/service.rs` |
| Request JSON body | 1 MiB | `tronc::json` |
| Image upload | the plan's `toile.upload_mb`, never over 100 MiB; PNG, JPEG, GIF, WebP only (SVG refused) | `modules/assets/upload.rs` |
| Link unfurl | `https` only, public addresses only (checked at every DNS answer and redirect), 3 redirects, 1 MiB, 5 s | `modules/cards/guard.rs` |
| Resolved card | cached 10 minutes per person and URL | `modules/cards/cache.rs` |
| Board document | see [board-schema.md](board-schema.md) | enforced once rooms ship |

Back to the [documentation index](README.md).
