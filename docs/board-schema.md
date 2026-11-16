# Board schema

Schema 1 is now specified by planche's
[spec/FORMAT.md](https://github.com/FacileStudio/planche/blob/main/spec/FORMAT.md) under the
`board` profile. planche is the canvas format Toile, Diapo and Clap share, and the server
validates every board with `planche::schema` and `Profile::Board`. When this page and the spec
disagree, the spec wins. A change to the board profile is a schema version bump.

## Board document contract (schema 1), summary

The client mirrors it in `apps/client/src/lib/board/schema.ts`.

- `meta`: Y.Map `{ schema: 1 }`.
- `shapes`: Y.Map<id, Y.Map>. Common keys: `type`, `x`, `y`, `w`, `h`, `rot`, `z` (fractional
  index string), `parent` (frame id or absent), `style` (plain object: `fill`, `stroke`, `width`,
  `font`, `size`, `align`), `locked`.
- Types: `sticky`, `text`, `rect`, `ellipse`, `diamond` (text in `text`: Y.Text), `line`,
  `arrow` (`points`, `from`/`to`: `{ shape, anchor: [0..1, 0..1] }` bindings, `head`), `ink`
  (`points`: flat number array `[x, y, pressure, ...]`, relative to `x`,`y`), `image` (`asset`:
  Toile asset id), `card` (`url`, cached `title`, `kind`, and the cached preview `image`,
  `description`, `favicon`, `site`), `frame` (`name`).
- Limits checked by the server: 20,000 shapes, 10,000 ink points per stroke, 10,000 chars per
  text, 1 MiB per update, numbers finite and within ±1e7, unknown types or keys refused.
- Presence (awareness, never stored): `user {name, color, picture?, guest}`, `cursor`,
  `selection`, `viewport` (for follow mode).
- Comments are NOT in the Y.Doc: Postgres rows anchored to `{shape?, x, y}`, broadcast to the room
  as a custom message so open clients refresh.

Back to the [documentation index](README.md).
