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
- A route that acts in another suite app for the caller (through Registre token exchange)
  answers `412 failed_precondition` when the person's Facile session behind it expired; the
  client then asks them to sign in again. Signing out drops every exchanged token.
- A plan limit is `402 payment_required` with `upgrade_url` (muse `planLimitFrom` reads it).

## Types

`Board`:

```json
{"id": "uuid", "name": "Rétro", "space_id": "string|null", "role": "owner|editor|commenter|viewer",
 "created_at": "RFC 3339", "updated_at": "RFC 3339", "thumbnail_url": "string|null",
 "deleted_at": "RFC 3339, only in the trash list"}
```

`role` for a signed-in caller: `owner` for a personal board's owner and a space `admin` or
`owner`, `editor` for a space `member`. Through a share link: `viewer` (view), `commenter`
(comment), `editor` (edit). `thumbnail_url` is `/api/boards/{id}/thumbnail.png?v=<unix time>` once
a thumbnail is stored, else `null`.

`Share` (answered once by `POST`): `{"id", "token", "url": "/s/<token>", "role": "view|comment|edit"}`.
The list answers `{"id", "role", "created_at"}`: only the token's hash is stored, so a lost link
cannot be shown again; create a new one.

`Comment`:

```json
{"id": "uuid", "parent": "uuid|null", "anchor": {"shape": "id (optional)", "x": 0, "y": 0},
 "body": "text", "author": "display name", "guest": false, "mine": true,
 "created_at": "RFC 3339", "updated_at": "RFC 3339", "resolved_at": "RFC 3339|null"}
```

## Live boards (WebSocket)

`GET /api/boards/{id}/ws` (cookie or bearer, `viewer` and up) and
`GET /api/share/{token}/ws?name=<guest name>` speak the y-websocket protocol: sync steps, updates
and awareness. Owners and editors (and `edit` links) send updates; viewers and commenters send
presence only, their updates are dropped. The server overwrites every presence state's
`user.name` (profile name, or the guest name) and `user.guest`; a connection may announce at
most 8 Yjs clients and never another connection's.

A guest name is trimmed, 1 to 40 characters, with no control characters, else 400 before the
upgrade.

Every update is checked against schema 1 ([board-schema.md](board-schema.md)); a refused one is
undone and its sender closed. Close codes: `1008` invalid update, revoked link, logged-out session,
lost space membership, or trashed board; `1009` update over 1 MiB or frame over 2 MiB; `1011`
server error; `1013` server restarting (reconnect) or a client too slow to keep up; `1001` a
bearer-JWT socket after 15 minutes (reconnect).

Comment changes reach open sockets as y-websocket message type `100` whose payload is the
string `{"type":"comments-changed"}` (lib0 `writeVarUint(100)` then `writeVarString(json)`):
refetch the comments.

### How the server reads schema 1

`docs/board-schema.md` stays normative; these are the readings it leaves open:

- Roots: only `meta` and `shapes`, both maps; any other root is refused. `meta` holds only
  `schema: 1` (absent is fine; new boards start with it).
- A shape is a `Y.Map` with a string `type`, and numbers `x` and `y`; every other key is optional.
- Numbers: `x`, `y`, `w`, `h`, `rot`, `style.width`, `style.size`, points and anchors are finite
  and within ±1e7. Strings (`z`, `parent`, `head`, `asset`, `url`, `title`, `kind`, `name`,
  `style.fill|stroke|font|align`, binding `shape`) are plain strings of at most 10,000 characters.
  `locked` is a boolean.
- `style`, `from`, `to` and `points` are plain values (not Y types). `from`/`to` are exactly
  `{shape, anchor: [a, b]}` with `a`, `b` in 0..1. `points` is a flat number array: pairs for
  `line`/`arrow`, triples for `ink`, at most 10,000 points.
- `text` (on `sticky`, `text`, `rect`, `ellipse`, `diamond`) is a `Y.Text` of at most 10,000
  characters.
- An absent key means unset: do not write `null` or `undefined`.

`GET /api/board-schema` answers the version, types and limits.

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
| `GET` | `/boards?space_id=&trash=true` | The scope's trash; needs `owner` of the scope, else 403 |
| `PATCH` | `/boards/{id}` | `{"name"}`, needs `editor`; answers the `Board` |
| `DELETE` | `/boards/{id}` | Moves to the trash (204), needs `owner`; open sockets close with 1008. `?permanent=true` deletes a trashed board for good |
| `POST` | `/boards/{id}/duplicate` | Optional `{"name"}` (default `<name> (copie)`): copies the live content into a new board of the same scope, 201; needs `editor` in the scope |
| `POST` | `/boards/{id}/restore` | Takes a trashed board back, needs `owner`; trashed boards are purged after 30 days |
| `GET` | `/boards/{id}/thumbnail.png` | The stored thumbnail (`image/png`), 404 until one exists |
| `GET` | `/boards/{id}/ws` | The y-websocket socket (see above) |
| `GET` | `/board-schema` | `{"schema": 1, "types", "limits"}`, no auth |
| `GET` | `/boards/{id}/shares` | Live links, needs `owner` |
| `POST` | `/boards/{id}/shares` | `{"role": "view|comment|edit"}`, 201 with the `Share`; needs `owner` |
| `DELETE` | `/boards/{id}/shares/{share}` | Revokes the link (204); its sockets close with 1008 |
| `GET` | `/share/{token}` | Public: `{"board": Board, "role": "view|comment|edit"}`; a revoked link or trashed board is 404 |
| `GET` | `/share/{token}/ws?name=` | Public: the guest socket |
| `GET` | `/boards/{id}/comments` | Every comment, oldest first; needs `viewer` |
| `POST` | `/boards/{id}/comments` | `{"anchor", "body", "parent"?}`, 201; needs `commenter`. A reply names a top-level `parent` of the same board and may leave out `anchor` (it takes the parent's) |
| `PATCH` | `/boards/{id}/comments/{comment}` | `{"body"?, "resolved"?}`: the body by its author only, `resolved` by any commenter |
| `DELETE` | `/boards/{id}/comments/{comment}` | Its author or an `owner`; replies go with it |
| `GET` | `/share/{token}/comments` | Public: the comments a link holder sees |
| `POST` | `/share/{token}/comments` | Public: `{"name", "anchor", "body", "parent"?}`, needs a `comment` or `edit` link (else 403) |
| `POST` | `/boards/{id}/assets` | Multipart `file`: 201 `{"asset", "url", "w", "h", "mime", "size"}`; needs `editor`. Stored in the uploader's Toile app folder in Nuage (`boards/<board>/<asset>.<ext>`). Over the plan's `toile.upload_mb`: 402 with `upgrade_url`, `key` and `limit`; not PNG, JPEG, GIF or WebP (SVG included): 400 |
| `GET` | `/boards/{id}/assets/{asset}` | The image bytes, read from Nuage with the uploader's token; needs `viewer`. `ETag` (sha-256), `Cache-Control: private, max-age=31536000, immutable`, 304 on `If-None-Match`. 503 while the uploader's Facile session is expired |
| `GET` | `/share/{token}/assets/{asset}` | Public: the same for a link holder (20 a second per IP, burst 200) |
| `POST` | `/share/{token}/assets` | Public: always 403, guests cannot upload |
| `POST` | `/cards/resolve` | `{"url"}` (http or https, at most 2048 characters): `{"kind", "title", "subtitle"?, "status"?, "icon", "url"}`. See "Live cards" |
| `POST` | `/boards/{id}/shapes/{shape}/task` | `{"project": <Opus project id>}`: creates an Opus task from sticky `shape` (title = its first line, description = its text) for the caller and writes a `card` shape (`kind: opus`) right of the sticky through the room. 201 `{"task": {"id", "title", "url"}, "card": "<shape id>"}`; needs `editor`. Not a sticky or empty: 400; no such shape or Opus project: 404 |
| `GET` | `/me/entitlements?space_id=` | `{"scope": "personal|space", "entitlements": {"ai.access": bool, "toile.upload_mb": n}, "upgrade_url"}`. No `space_id` is the caller's own plan; a space they are not in is 404. `-1` is unlimited |
| `POST` | `/boards/meeting` | `{"room", "name"?, "space_id"?}`: the Echo room's board and edit link (see below); 201 when made, 200 after |
| `GET` | `/mcp/actions` | The caller's agent actions awaiting confirmation, each with `state` (`ready|changed|missing`) |
| `POST` | `/mcp/actions/{id}/confirm` | Runs the action: `{"status": "done"}`; an agent token (with an actor) gets 403 |
| `POST` | `/mcp/actions/{id}/dismiss` | Refuses the action, 204 |

### Live cards

`kind` is `docs`, `feuilles`, `opus`, `agenda`, `nuage` or `link`; `icon` is the same key. A link
whose origin is a suite app's `<APP>_URL` is read from that app with the caller's own exchanged
token: Docs `/d/{id}`, Feuilles `/w/{id}`, Opus `/dashboard/workspace/{w}/project/{p}/task/{id}`
(with its `status`), Agenda `/events/{id}` or `?event={id}` (`subtitle` is `start_at`). Any other
page of those apps, an item the caller cannot open, and every Nuage link (Toile only has
app-folder access) answer a card titled with the host. Other links are unfurled from their Open
Graph tags (`og:title` or `<title>`, `og:description` as `subtitle`) under the SSRF guard: `https`
only, public addresses only at every DNS answer and redirect, 3 redirects, 1 MiB, 5 s; anything
it refuses is a host-only `link` card. Cards are cached per person and URL for 10 minutes.

Public share routes allow 2 requests a second per client IP (burst 30); guest comment posts 1
every 2 seconds (burst 10); past that, 429.

## Echo meeting boards

`POST /api/boards/meeting` gives an Echo room its board. Echo calls it as the room's host, with
that person's bearer token (a Registre token exchanged for Toile, or a Toile session token).

```json
{"room": "echo room id", "name": "optional, first call only (default Réunion)", "space_id": "optional, first call only"}
```

It answers `{"board": Board, "edit_link": "/s/<token>", "created": true|false}`. The pair
(caller, `room`) names one board: later calls answer the same board and the same edit link, while
both live. If the host trashed the board or revoked the link, the next call makes a new board
(201). `room` is trimmed, 1 to 200 characters, no control characters. The link is relative to
`https://toile.facile.studio`: participants open it in a browser, or a client resolves it with
`GET /api/share/{token}`. Toile keeps the link sealed with `PORTE_TOKEN_KEY` to answer it again.

## MCP

`/mcp` (outside `/api`) is the remote MCP server: Streamable HTTP, stateless (spec
2026-07-28), behind Registre tokens bound to `https://toile.facile.studio/mcp`, scopes
`toile.read` and `toile.write`. It is free for every account. A first call creates the local
account. Metadata: `/.well-known/oauth-protected-resource/mcp`.

| Tool | Scope | What it does |
|---|---|---|
| `toile_list_boards` | read | `{space_id?}`: the boards of one scope, or personal plus every space; and the caller's spaces |
| `toile_read_board` | read | `{board_id}`: every shape as one line (`type id at x,y wxh color: "text"`), frames followed by their children, then `outside frames:`; arrows name the shapes they join. Plus the board, SVG export and thumbnail URLs |
| `toile_add_shapes` | write | `{board_id, shapes}`: sticky, text, rect, ellipse, diamond, frame, arrow, line. Each takes an optional `ref`, `text`, `x`, `y`, `w`, `h`, `color`, `frame`, and for arrows `from`/`to` (an id or a `ref`). Shapes without `x`/`y` go inside their frame, or in rows to the right of the content; a new frame sizes to its new children. At most 500 per call |
| `toile_update_shapes` | write | `{board_id, shapes: [{id, text?, x?, y?, w?, h?, color?, frame?}]}`, all or none; moving a frame moves its contents; `frame: ""` takes a shape out of its frame |
| `toile_delete_shapes` | write | `{board_id, ids}`; a frame takes its contents. Confirmed by the person first: an elicitation when the client can ask, otherwise `pending_confirmation` with a `confirm_url` to `/pending?action=<id>` |
| `toile_comment` | write | `{board_id, body, shape_id? \| x?, y? \| reply_to?}`, signed by the caller |

Every write runs in the board's room (`Hub::write`): open clients see it live and schema 1 checks
it like any client update. Access is the caller's own: viewers read, commenters comment, editors
and owners change shapes.

Back to the [documentation index](README.md).
