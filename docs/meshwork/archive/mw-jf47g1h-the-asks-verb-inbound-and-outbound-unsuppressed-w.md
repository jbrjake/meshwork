---
id: mw-jf47g1h
title: The asks verb — inbound and outbound, unsuppressed, with answered-by state and age
category: capability/asks
needs: [mw-xg67266, mw-21qzw9n]
seq: 690
verify: run cargo test e2e::asks_verb
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-1-2-verbs
  - docs/PROPOSAL-analytics.md#§-4-6-asks
status: done
created: 2026-09-07T16:32Z
---
Typed as `addressed`/`inbox`/`portfolio show`/`list` 22 times and never existed. With R-D item
D.2: one canned query over `asks` at union scope (the same read the inbox performs), text and
`--json`, in and out sections, each row `gid · age · answered-by <gid> (<status>)`; the `prime`
inbox line names it as the expanding verb. `cli_surface_frozen` re-blessed.

## log
- 2026-09-07T16:32Z created
- 2026-09-22T13:56Z open→doing — claimed by claude (25ed6f72-cf72-4c6f-9164-86d9af1be3d2)
- 2026-09-22T14:01Z doing→done — verify exit 0 @ cd5a9c2+4
