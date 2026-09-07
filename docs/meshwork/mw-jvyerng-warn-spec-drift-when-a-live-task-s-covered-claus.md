---
id: mw-jvyerng
title: Warn spec-drift when a live task's covered clause no longer hashes to its pin
category: capability/spec
needs: [mw-psbn61z]
seq: 750
verify: run cargo test lint::spec_drift
docs:
  - docs/PROPOSAL-spec-traceability.md#§-b-a-covers-edge
status: open
created: 2026-09-07T16:32Z
---
Fix is explicit re-pin, never silent. Done tasks with drifted pins are not reopened; they are
`spec audit`'s re-open candidates. The weft reads pinned edges as exact and computes the same
predicate, so the two never disagree on a pin. MW-T6.

## log
- 2026-09-07T16:32Z created
