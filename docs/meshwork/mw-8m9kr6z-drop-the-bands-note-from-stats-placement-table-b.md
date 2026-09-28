---
id: mw-8m9kr6z
title: "Drop the bands note from stats' placement table — bands were rejected, so stats must stop promising them"
category: capability/stats
seq: 145
relates: [mw-ryd25rq, mw-549rh9w]
docs:
  - docs/DESIGN-meshwork.md#6-cli-surface-complete-for-v1--anything-not-here-is-a-non-goal
  - docs/REQUIREMENTS-meshwork.md#3-non-goals-normative--this-list-is-the-anti-jira-anti-nerdsnipe-contract
verify: "all(lacks src/cli/stats.rs /band/, lacks docs/DESIGN-meshwork.md /waits on bands/, run cargo test target=suite e2e::stats_tables)"
status: open
created: 2026-09-28T15:27Z
---
`meshwork stats` ends its placement table with
`placed by: seq or unranked — bands are not built`. That reads as a roadmap
promise, and bands were rejected: the owner's 2026-09-20 ruling struck "bands
over categories, declared in a per-repo config file" as the prioritization
mechanism, and REQUIREMENTS §3 records the rejection. A user reading stats is
told a feature is coming that will not come.

Three places carry it, and they move together:

- `src/cli/stats.rs` `render()`, the `"placement"` arm, pushes the note.
- DESIGN §6's `stats` row specifies it: "with a note that `placed_by` waits on
  bands (R-C)". The spec line goes in the same commit as the code.
- `tests/suite/e2e_stats.rs` `stats_tables` asserts `text.contains("placed by:")`
  and nothing about the wording, so it never caught this. Make it assert the note
  names no band.

What replaces it: a note that says what places a task today, in plain words, or
no note at all. It must not name a mechanism that does not exist, and it must not
cite a ruling id or section (user-facing output carries no internal citations).
When the prioritization rubric lands (mw-ryd25rq's ruling, then the rewrite of
mw-ex5x0y2), `placed by` becomes the rubric's term string; that is not this task.

## log
- 2026-09-28T15:27Z created
