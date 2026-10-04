---
id: mw-53wgsbs
title: Make prime's ready count agree with ready — an open ask counts as ready in the weather but is never listed
category: core/render
docs: [docs/REQUIREMENTS-meshwork.md#l-the-session-ritual]
verify: run cargo test prime_ready_count_excludes_outbound_asks
status: open
created: 2026-09-24T01:13Z
handoff: |
  Not a one-file change, which is why it waited. `pulse.ready_n`
  (src/pulse.rs, `count(&rows, |r| r.ready)`) counts `GraphRow.ready`; the
  row carries no `to:`, so the exclusion needs the task facts (`mine:
  Vec<&TaskFacts>` in `pulse::compute`) or a new row column. The SQL twin
  is `p_graph` in FORMAT-views.sql (`count(*) FILTER (WHERE ready) AS
  ready_n`), generated from FORMAT.md §Views by `scripts/mine_views.py
  --sql` and registered verbatim, so the Rust and SQL counts must change
  together: edit FORMAT.md's view text, regenerate, then match in Rust.
  The conformance corpus pins `pulse` output (tests/suite/conformance.rs
  row ("pulse", "repo")), so any fixture with an open outbound ask changes
  its golden — `--bless` and review the diff. tables.rs checks only the
  status sums of the SQL pulse. Red test: a store with one open ask (`to:`
  set) and one ready task; prime's weather must say `ready 1 of 2 open`,
  and the SQL `pulse.ready_n` must agree. The smaller alternative the task
  body names — changing `graph.ready` — touches the views contract and
  every ready consumer; the pulse-only exclusion with a matching SQL
  filter is the smaller change.
---
MW-L5 keeps a task carrying `to:` out of its own repo's `ready` and `next`. The weather's `ready N of M open` counts it anyway: `pulse.ready_n` counts `graph.ready` (`src/pulse.rs`), and `graph.ready` is true for an open ask with no unmet needs.

Observed in a replay of the README demo after `dep add sa-38wd6se --needs sa-z1ecwc8`: prime printed `queue: ready 2 of 5 open` while `ready` listed one task (the other was the ask, listed under `asks out`).

Decide where the exclusion belongs. `pulse.ready_n` alone is the smaller change. Changing `graph.ready` touches the FORMAT.md views contract and the conformance corpus. The test pins that an open outbound ask is not counted.

## log
- 2026-09-24T01:13Z created
- 2026-10-04T15:02Z handoff by claude (70b8c3c2-a3ae-4cee-81aa-681b84b44405)
