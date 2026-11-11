# Architecture

How a request moves through the Toile API, who may see a board, and what the database holds.
The v1 target (rooms, sharing, export, integrations) is in [plan.md](plan.md); this page
describes what the code does today.

```
Internet ──▶ Traefik ──▶ api (:4000, one replica) ─┬──▶ /health, /ready, /version, /docs
                                                   ├──▶ /api/*          auth + modules
                                                   └──▶ everything else the SPA (apps/client/build)
                                                              │
                                                         Postgres 16
```

## Components

| Path | Role |
|---|---|
| `src/server.rs` | Boot: env, logger, pool, `migrate` subcommand, migrations, kits, loops, router, drain |
| `src/auth/` | porte wiring: sessions, Registre sign-in, the spaces store, directory apply, GDPR erase |
| `src/modules/boards/` | Board list, create and read, and the one `access()` function |
| `src/modules/spaces/` | `GET /api/spaces`: the caller's spaces and roles |
| `src/apiref.rs` | The OpenAPI document served at `/docs` |
| `src/security.rs` | The CSP script hashes of the built client |

## Access

Every board route calls `access::access(state, user, board)`:

- a personal board (`space_id IS NULL`) is `manager` for its owner and invisible to everyone else;
- a space board is `editor` for a `member`, `manager` for an `admin` or `owner`, invisible
  otherwise;
- a deleted board is invisible; an instance admin gets no bypass; invisible always answers `404`,
  never `403`.

A space member is matched by Registre subject: `space_members.subject` joins
`porte_identities.subject` where `provider` is `OIDC_ISSUER`. Lists resolve the scope with
`access::scope` and filter in the same SQL statement.

## Directory

With `OIDC_ISSUER` set, porte's `Syncer` pulls Registre's directory snapshot (and Registre nudges
`POST /api/registre/sync`). `auth::SpacesApply` replaces `spaces` and `space_members` in one
transaction, once per snapshot version. A space missing from a snapshot is marked deleted and its
boards with it; an empty snapshot is refused while spaces exist.

`POST /api/registre/deletions` and the deletion loop erase a person: their personal boards,
their memberships, then their `porte_users` row. Space boards stay with the space.

## Data model

| Table | Holds |
|---|---|
| `porte_*` | porte's schema (`0001_porte.sql`): users, identities, sessions, login codes |
| `spaces`, `space_members` | The Registre directory, by subject |
| `directory_state`, `deletion_state` | The directory version and the deletion cursor |
| `boards` | `id`, `owner_user_id`, `space_id` (null for personal), `name`, `created_at`, `updated_at`, `deleted_at` |

Back to the [documentation index](README.md).
