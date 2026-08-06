---
id: mw-0f4j
title: "Field setters: --seq/--docs on add + meshwork set verb (README spec)"
status: done
category: meta/readme
verify: cargo test e2e::field_setters
seq: 230
docs:
  - DESIGN-meshwork.md#§-6-cli-surface
  - DESIGN-meshwork.md#§-7b-prime-as-materialized-handoff
created: 2026-08-06
---
README: "Hand-editing is legal and expected but never necessary" —
transcripts show `add --seq` and `meshwork set <id> --handoff "…"`. Today
add lacks --seq/--docs and no verb edits fields on existing tasks.
Frozen-surface change: extend the §6 verb table (add flags + `set` verb
for scalar frontmatter fields incl handoff:) and e2e::cli_surface_frozen
in the same commit. Owner re-ruled 2026-08-06 via README, superseding
§7b/§15.7 "hand-edit only, no verb" for handoff: — amend those clauses
when this lands, and drop the README footnote's setter exception.

## log
- 2026-08-06 created
- 2026-08-06 open→done — verify exit 0
