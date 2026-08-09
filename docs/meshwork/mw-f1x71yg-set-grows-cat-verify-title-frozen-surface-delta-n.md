---
id: mw-f1x71yg
title: "set grows --cat/--verify/--title (frozen-surface delta — needs the §6 ruling)"
status: open
category: core/lifecycle
verify: cargo test e2e::set_cat_verify
discovered-from: mw-ntt5
seq: 250
docs:
  - DESIGN-meshwork.md#§-6-cli-surface
created: 2026-08-07T13:47Z
---
Pilot evidence (sazed), both sessions. Migration: enriching 94 imported
tasks with categories and verifies — the NORMAL path after any import —
had no CLI route; a Python script rewrote frontmatter directly. Work
session: THREE python hand-edits of store files in 65 minutes — retitle
(after `set --seq 1 --title "…"` was rejected wholesale, losing the seq
change with it, and the task stayed mis-ranked), a verify replacement
(the vacuous→red fix), and a body repair. `set` §6 purpose is "field
edits without opening the file"; --cat/--verify/--title are field edits.
Surface is frozen, so this files as a proposal: OWNER RULING REQUIRED
before any code. If rejected, the playbook should document the hand-edit
as sanctioned instead.

## log
- 2026-08-07T13:47Z created

## comments
- 2026-08-08T16:42Z [claude] Field evidence (sazed, first 8 post-migration sessions, reviewed 2026-08-08): set --verify and set --title were attempted in 3 sessions and rejected (v0.1.5 surface), each falling back to a python3 rewrite of the task file; in one session the rejected set --verify was followed by a close that ran the STALE verify (exit 1) before the hand-edit landed. Sessions keep 'set --help | head' as a reflex. Strengthens the §6 case.
- 2026-08-09T23:35Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Field evidence (sazed, 2026-08-09, session f1ee9642): ninth session, set --verify rejected again. The agent checked set --help, then fell back to a 15-line python script to rewrite the verify: line across two task files — hand-rolling the exactly-one-verify-line assertion the verb would own. The fallback for a missing one-line field edit is now a program.
