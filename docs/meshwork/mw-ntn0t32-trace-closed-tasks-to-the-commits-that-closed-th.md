---
id: mw-ntn0t32
title: Trace closed tasks to the commits that closed them
status: open
category: core/lifecycle
verify: cargo test e2e::commit_trace
docs:
  - REQUIREMENTS-meshwork.md#§-e-lifecycle-discipline
  - DESIGN-meshwork.md#§-6-cli-surface
created: 2026-08-07T00:27Z
---
Owner-requested 2026-08-07: a mechanized, automated tie between a
closed task and the commits whose work closed it — "what work went
into closing this" must be answerable without archaeology. Today the
link is convention only (commit subjects carry the id, e.g.
"feat(lifecycle): … (mw-tb6gdr9)"); nothing records or derives it.

Constraints that shape the fix: meshwork never installs git hooks
(MW-A3 boundary — commit-msg enforcement is out) and the CLI surface
is frozen (§6 — display must ride on existing verbs unless an owner
ruling adds surface). Candidate mechanics, decided at implementation:
(a) close-side: `close` stamps the repo HEAD (+ dirty marker) into the
`→done` log line — cheap, but the closing commit itself lands AFTER
close runs, so HEAD names the parent, not the closing commit;
(b) read-side: derive the commit list with `git log --grep=<id>` over
subjects/bodies — zero write-path change, leans on the existing
id-in-subject convention, works retroactively for every task already
closed this way; surfaced in `show` (a "commits:" tail) and/or a §15.3
recipe. Likely both: (a) anchors the moment, (b) recovers the set.
Lint could warn when a done task's id appears in no commit message —
that makes the convention self-enforcing without hooks.

## log
- 2026-08-07T00:27Z created
