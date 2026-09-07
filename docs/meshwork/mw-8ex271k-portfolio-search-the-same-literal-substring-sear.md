---
id: mw-8ex271k
title: portfolio search — the same literal substring search across every registered store
category: core/query
needs: [mw-xg67266]
seq: 700
verify: run cargo test e2e::portfolio_search
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-2-4-discoverability-tax
status: open
created: 2026-09-07T16:32Z
---
Prior art in a sibling store is invisible to per-repo `search`; the lab replaced `prime` with a
hand-rolled cross-repo query for lack of it. With R-D item D.3: the same canned `strpos` SQL
over the union, hits grouped by repo, live before terminal, the MW-D2 cap + `--all`. Never
prunes. `cli_surface_frozen` re-blessed.

## log
- 2026-09-07T16:32Z created
