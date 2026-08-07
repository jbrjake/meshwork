---
id: mw-pvfrpd4
title: "mirror: default-branch guard (append-only is unretractable)"
status: done
category: core/mirror
verify: cargo test e2e::mirror_branch_guard
docs:
  - DESIGN-meshwork.md#§-8-github-push
  - REQUIREMENTS-meshwork.md#§-h-github-push
seq: 75
created: 2026-08-06
---
Owner-accepted 2026-08-06 (format-hardening review; DESIGN §8
amendment — it is currently silent on branches). The store rides
branches like code (correct), but mirror is append-only and
UNRETRACTABLE: a push from a feature branch publishes issues/comments
for state that may rebase away or never merge. Rule: mirror push
refuses off the repo's default branch, names the branch, exits
nonzero; a config override (mirror allow_non_default) stays loud in
output. One rule, one test; must be decided before 3.1 builds the
create path (mw-cvw8 needs this).

## log
- 2026-08-06 created
- 2026-08-07T02:47Z open→doing — claimed by claude
- 2026-08-07T03:02Z doing→done — verify exit 0

## comments
- 2026-08-07T02:48Z [claude] Landed ahead of 3.1 as ruled: branch_guard() in cli/stubs.rs fires on mirror push before the M3 stub error — refuses off-default naming both branches; default = local origin/HEAD symbolic ref (zero network; tests set it with git symbolic-ref, no remote needed); unset origin/HEAD = indeterminate = refuse naming the set-head fix. config.mirror widened to bool-or-[mirror]-table (untagged serde); [mirror] allow_non_default = true skips the guard but prints the override + branch every run. DESIGN §8 gains the branch-guard paragraph. When 3.1 builds the real push (mw-cvw8), the guard is already in its path.
