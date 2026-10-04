---
id: mw-pxm63n8
title: Stop implicit-edge firing on the ids that lint --fix's own log lines mention
status: done
category: core/hygiene
discovered-from: mw-6k73vyj
verify: run cargo test implicit_edge_ignores_fix_log_lines
docs:
  - FORMAT.md#mentions-every-id-named-anywhere-resolved
created: 2026-10-04T16:29Z
---

After `lint --fix` repairs a post-merge duplicate id, the re-slugged file's log reads `re-slugged from <old>` and each rewritten referencer's log reads `<key> now <new>, was <old>`. Both are live mentions of a live task with no edge behind them, so the very next `lint` folds them into `implicit-edge` (`2 edgeless live mentions` in a scratch repo on 2026-10-04, right after the repair that mw-6k73vyj landed). Machine-written provenance is never a missing edge, and a fix that leaves a fresh warning teaches the reader to skim the folded line.

Two places the exclusion could live. The `mentions` view is FORMAT.md contract — prose, `FORMAT-views.sql`, conformance goldens — so excluding `lint --fix:` log entries there is a view change with a `--bless` and a reviewed diff. `lint_views::implicit_edges` is lint-only: it could drop a pair whose only mentions sit in `log` entries that `lint --fix` wrote, with no contract change. Decide which, write the red test first, and keep the view's row count honest either way.

## log
- 2026-10-04T16:29Z created
- 2026-10-04T16:35Z open→doing — claimed by claude (2a6ba2e9-aa96-4775-b0f1-2a5cd137c964)
- 2026-10-04T16:42Z doing→done — verify exit 0 @ 78d8447+2
