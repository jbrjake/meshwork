---
id: mw-ntn0t32
title: Trace closed tasks to the commits that closed them
status: done
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
- 2026-08-07T03:06Z open→doing — claimed by claude
- 2026-08-07T03:11Z doing→done — verify exit 0 @ 1359b45+9

## comments
- 2026-08-07T03:10Z [claude] Landed both halves as the body suggested. (a) close-side: head_anchor() appends ' @ <short-sha>[+N]' to →done notes (verify-0 AND waive paths; N = git status --porcelain line count repo-wide) — unborn HEAD omits silently (mw-3jwwh5d precedent). Anchor is note text under the log grammar, a convention not a parse rule (FORMAT.md updated). (b) read-side: show gains a commits: tail — git log -F --grep=<id> (stuck form; the separated form silently fails), %h %s, capped 10 with the … marker, JSON carries commits[] + commits_total; empty = omitted. (c) the lint warn was deliberately NOT added: fixture/test repos are freshly git-init'd, so every done task in every corpus copy would warn — noise that would train people to ignore lint. The gap is already visible as a missing commits: tail in show; revisit only if real repos show the convention slipping.
