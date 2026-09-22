---
id: mw-s0ptwn0
title: Say in prime's weather how many live tasks the spec moved under
category: capability/spec
needs: [mw-jvyerng]
seq: 770
verify: run cargo test prime_spec_drift_line
docs:
  - docs/PROPOSAL-spec-traceability.md#§-c-coverage-and-rulings
status: done
created: 2026-09-07T16:32Z
---
One weather line, omitted at zero: `spec moved under 3 live tasks (ids…)`. In Rust over the
`covers` rows `prime` already parsed; byte-clamped like every line. MW-T10.

## log
- 2026-09-07T16:32Z created
- 2026-09-22T14:29Z open→doing — claimed by claude (25ed6f72-cf72-4c6f-9164-86d9af1be3d2)
- 2026-09-22T14:33Z doing→done — verify exit 0 @ 6f5f133+5
