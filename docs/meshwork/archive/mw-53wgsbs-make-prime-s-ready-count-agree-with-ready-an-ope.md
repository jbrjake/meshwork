---
id: mw-53wgsbs
title: Make prime's ready count agree with ready — an open ask counts as ready in the weather but is never listed
category: core/render
docs: [docs/REQUIREMENTS-meshwork.md#l-the-session-ritual]
verify: run cargo test prime_ready_count_excludes_outbound_asks
status: done
created: 2026-09-24T01:13Z
---
MW-L5 keeps a task carrying `to:` out of its own repo's `ready` and `next`. The weather's `ready N of M open` counts it anyway: `pulse.ready_n` counts `graph.ready` (`src/pulse.rs`), and `graph.ready` is true for an open ask with no unmet needs.

Observed in a replay of the README demo after `dep add sa-38wd6se --needs sa-z1ecwc8`: prime printed `queue: ready 2 of 5 open` while `ready` listed one task (the other was the ask, listed under `asks out`).

Decide where the exclusion belongs. `pulse.ready_n` alone is the smaller change. Changing `graph.ready` touches the FORMAT.md views contract and the conformance corpus. The test pins that an open outbound ask is not counted.

## log
- 2026-09-24T01:13Z created
- 2026-10-04T15:02Z handoff by claude (70b8c3c2-a3ae-4cee-81aa-681b84b44405)
- 2026-10-04T15:34Z open→doing — claimed by claude (5566559a-b6fd-4780-8c73-3214f8c33f5d)
- 2026-10-04T15:46Z doing→done — verify exit 0 @ 506dc44+4
