---
id: mw-qf0bsb6
title: "Accept in add --batch the plain values the flags accept — a `: ` inside a title or verify, or a scalar in a list slot, no longer refuses the whole batch"
category: cli/add
seq: 120
docs:
  - docs/DESIGN-meshwork.md#§-6-cli-surface
  - src/cli/add_batch.rs
verify: run cargo test e2e::batch_plain_scalars_quoted_like_add
status: done
created: 2026-09-23T18:32Z
---
Three sazed batches in two days refused with `mapping values are not allowed in this
context at line 9 column 110 — nothing written`. Every one was a plain-scalar value
carrying `: ` — `verify: … grep -qE "test result: ok…"`, `verify: grep -q 'AXIS: BOTH' …`
— which YAML reads as a nested mapping. A fourth session hand-fixed `needs: "@x"` to
`needs: ["@x"]` before the batch would take it. `add "title" --verify …` never has this
problem: `write::yaml_scalar` quotes what YAML would misread. The batch slot for `docs:`
already accepts the flag's scalar shape; extend that parity to every key.

Shape, in `add_batch::parse_entry`, top-level lines only: a scalar string key (title,
category, verify, to, answers, parent, discovered-from, blocked-reason, claimed-by,
waived, handoff, created) whose value is unquoted and not a block or flow indicator is
re-emitted through `yaml_scalar`; a list key (needs, relates, labels) whose value is a
bare scalar becomes a one-element flow sequence, as `docs:` does. `@handle` refs still
resolve afterwards. When a document still fails to parse, the error names the key on the
failing line. Numeric and enum keys (seq, github, status) pass through untouched.

Docs in the same commit: DESIGN §6 `add --batch` row, SKILL.md authoring bullet.

## log
- 2026-09-23T18:32Z created
- 2026-09-23T18:37Z open→doing — claimed by claude (c86c9e3d-7fc8-4611-b368-47218aec1288)
- 2026-09-23T18:43Z doing→done — verify exit 0 @ d2f9543+6
