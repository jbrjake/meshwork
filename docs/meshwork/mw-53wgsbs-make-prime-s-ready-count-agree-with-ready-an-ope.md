---
id: mw-53wgsbs
title: Make prime's ready count agree with ready — an open ask counts as ready in the weather but is never listed
category: core/render
docs: [docs/REQUIREMENTS-meshwork.md#l-the-session-ritual]
verify: run cargo test prime_ready_count_excludes_outbound_asks
status: open
created: 2026-09-24T01:13Z
---
MW-L5 keeps a task carrying `to:` out of its own repo's `ready` and `next`. The weather's `ready N of M open` counts it anyway: `pulse.ready_n` counts `graph.ready` (`src/pulse.rs`), and `graph.ready` is true for an open ask with no unmet needs.

Observed in a replay of the README demo after `dep add sa-38wd6se --needs sa-z1ecwc8`: prime printed `queue: ready 2 of 5 open` while `ready` listed one task (the other was the ask, listed under `asks out`).

Decide where the exclusion belongs. `pulse.ready_n` alone is the smaller change. Changing `graph.ready` touches the FORMAT.md views contract and the conformance corpus. The test pins that an open outbound ask is not counted.

## log
- 2026-09-24T01:13Z created
