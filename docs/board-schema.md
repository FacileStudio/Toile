# Board schema

Normative. This is the board document contract every track codes against, copied verbatim from
the v1 plan ([plan.md](plan.md)). When code and this page disagree, the code is wrong. A change
here is a schema version bump.

## Board document contract (schema 1)

Written first as `docs/board-schema.md`, mirrored by `apps/client/src/lib/board/schema.ts` and
`apps/api/src/modules/board/schema.rs`. Every track codes against it.

- `meta`: Y.Map `{ schema: 1 }`.
- `shapes`: Y.Map<id, Y.Map>. Common keys: `type`, `x`, `y`, `w`, `h`, `rot`, `z` (fractional
  index string), `parent` (frame id or absent), `style` (plain object: `fill`, `stroke`, `width`,
  `font`, `size`, `align`), `locked`.
- Types: `sticky`, `text`, `rect`, `ellipse`, `diamond` (text in `text`: Y.Text), `line`,
  `arrow` (`points`, `from`/`to`: `{ shape, anchor: [0..1, 0..1] }` bindings, `head`), `ink`
  (`points`: flat number array `[x, y, pressure, ...]`, relative to `x`,`y`), `image` (`asset`:
  Toile asset id), `card` (`url`, cached `title`, `kind`), `frame` (`name`).
- Limits checked by the server: 20,000 shapes, 10,000 ink points per stroke, 10,000 chars per
  text, 1 MiB per update, numbers finite and within ±1e7, unknown types or keys refused.
- Presence (awareness, never stored): `user {name, color, picture?, guest}`, `cursor`,
  `selection`, `viewport` (for follow mode).
- Comments are NOT in the Y.Doc: Postgres rows anchored to `{shape?, x, y}`, broadcast to the room
  as a custom message so open clients refresh.

Back to the [documentation index](README.md).
