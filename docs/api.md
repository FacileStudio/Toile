# API

Every route, from the routers in `apps/api/src`. The live reference is served at `GET /docs`
(Scalar) and `GET /docs/openapi.json`; `tests/lifecycle.rs` fails when a mounted route is missing
from it.

`/docs` sits at the root, as on every suite app, so the client must not use `/docs` as a page
path.

## Conventions

- Everything below is under `/api` unless it starts with another root.
- Auth is a porte session: the `session` cookie, with `X-Facile-CSRF: <any>` on every mutation,
  or `Authorization: Bearer <session token>`.
- Errors are `{"error":{"code","message"}}` with the suite codes (`invalid_argument` 400,
  `unauthenticated` 401, `permission_denied` 403, `not_found` 404, `internal` 500).
- Request bodies refuse unknown fields.
- A board the caller cannot see is always 404. Seen but not allowed is 403.

## Types

`Board`:

```json
{"id": "uuid", "space_id": "string|null", "name": "Rétro",
 "created_at": "RFC 3339", "updated_at": "RFC 3339", "access": "viewer|editor|manager"}
```

## Routes

| Method | Path | Answers |
|---|---|---|
| `GET` | `/health`, `/ready`, `/api/health`, `/api/ready` | Probes |
| `GET` | `/version` | `{"version", "board_schema"}` |
| `GET` | `/auth/config` | Whether sign-in is SSO only and OIDC is on |
| `GET` | `/auth/oidc` | 302 to Registre |
| `GET` | `/auth/oidc/callback` | Registre's redirect back; sets the session cookie |
| `POST` | `/auth/oidc/exchange`, `/auth/oidc/device/exchange` | CLI token exchanges |
| `POST` | `/auth/sync-profile` | Refreshes the profile from Registre |
| `POST` | `/auth/backchannel-logout` | Registre's back-channel logout |
| `POST` | `/auth/logout` | Ends the session |
| `GET` | `/auth/me` | The signed-in person, or 401 |
| `GET` | `/auth/session` | The signed-in person, or `{"user": null}` |
| `POST` | `/registre/sync` | Registre's directory nudge, 202 |
| `POST` | `/registre/deletions` | Registre's signed account deletion, 202 |
| `GET` | `/spaces` | `{"spaces": [{"id", "name", "role"}]}` |
| `GET` | `/boards?space_id=` | The live boards of a scope, newest change first. No `space_id` is the personal scope; a space the caller is not in is 404 |
| `POST` | `/boards` | `{"name", "space_id"?}`, 201 with the `Board`. `name` is trimmed, blank becomes `Sans titre`, over 200 characters is 400. Needs `editor` in the scope |
| `GET` | `/boards/{id}` | One `Board` |

Back to the [documentation index](README.md).
