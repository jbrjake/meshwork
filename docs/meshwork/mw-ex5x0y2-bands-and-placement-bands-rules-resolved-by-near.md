---
id: mw-ex5x0y2
title: Bands and placement — [bands] rules resolved by nearest ancestor, the composite key, placed_by in the projection
category: capability/rank
needs: [mw-zwgp6x7, mw-ryd25rq, mw-v4d2hwt]
seq: 600
verify: run cargo test e2e::ready_bands_placement
docs:
  - docs/PROPOSAL-prioritization.md#§-4-the-shape
  - docs/PROPOSAL-prioritization.md#§-6-surfaces
  - FORMAT.md#§-config-toml
status: open
created: 2026-09-07T16:32Z
---
`[bands]` in `config.toml`: category-prefix → band, `default`, one-line `reason`, resolved by
whole-segment nearest ancestor (`category_matches`). `place(t)` = `seq` else band else
`default`; the key `(overlay, place, -unlock, created, id)` with `-unlock` entering only when
`[bands]` exists (MW-R2's byte-identical promise otherwise — a golden test on every fixture
store). `placed_by` (`seq` | `band:<rule>` | `floor`) in the tasks projection and `placement`
in every ordering verb's `--json` (schema bump). FORMAT.md rows. MW-R1–R3.

## log
- 2026-09-07T16:32Z created
