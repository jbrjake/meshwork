---
id: mw-bd390q6
title: Accept --reason on drop, recorded to the log
status: open
category: core/lifecycle
verify: ./meshwork drop --help | grep -q -- --reason
docs:
  - DESIGN-meshwork.md#§-6-cli-surface
created: 2026-08-12T20:48Z
---
`block` demands a reason; `drop` refuses one. Agents guess the symmetry
exists and lose the reason from the structured record when it doesn't:
two sazed sessions ran `drop <id> --reason "…"`, got the usage error,
and split into comment-then-drop (a28a4d31) or a bare drop whose
rationale ("duplicate of sa-y94j76x") survives only in chat scrollback
(4b5a9264). Record it to the `## log` entry like block does. Surface
change — needs the DESIGN §6 owner ruling.
