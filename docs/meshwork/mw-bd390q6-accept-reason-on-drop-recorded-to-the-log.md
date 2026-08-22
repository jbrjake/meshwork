---
id: mw-bd390q6
title: Accept --reason on drop, recorded to the log
status: open
category: core/lifecycle
verify: ./meshwork drop --help | grep -q -- --reason
docs:
  - DESIGN-meshwork.md#§-6-cli-surface
created: 2026-08-12T20:48Z
seq: 190
blocked-reason:
handoff: |
  APPROVED 2026-08-22 (optional flag) — not yet worked, cleared for the
  next session. Implementation sketch: drop currently takes IdArg
  (src/cli/transition.rs); give it its own args struct with an optional
  --reason, and transition() already threads a reason into the log note
  for block — pass Some(reason) on drop the same way. DESIGN §6 drop
  row gains [--reason "text"]; check e2e::cli_surface_frozen for what pins
  the surface. Verify is the drop --help grep.
---
`block` demands a reason; `drop` refuses one. Agents guess the symmetry
exists and lose the reason from the structured record when it doesn't:
two sazed sessions ran `drop <id> --reason "…"`, got the usage error,
and split into comment-then-drop (a28a4d31) or a bare drop whose
rationale ("duplicate of sa-y94j76x") survives only in chat scrollback
(4b5a9264). Record it to the `## log` entry like block does. Surface
change — needs the DESIGN §6 owner ruling.

## log
- 2026-08-22T01:22Z open→blocked — awaiting DESIGN §6 owner ruling — --reason on drop is a frozen-surface change; evidence for it is in the task body
- 2026-08-22T01:43Z blocked→open

## comments
- 2026-08-22T01:43Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Owner ruling 2026-08-22: APPROVED, optional. §6 drop row gains [--reason "text"]; the reason lands in the →dropped log note like block's. Bare drop keeps working — terminal drops are sometimes self-evident.
