---
id: mw-chcqk6g
title: Prune satisfied entries out of sequence.md
category: plan/m2
seq: 50
needs: [mw-2nmsys2]
verify: cargo test e2e::portfolio_sequence_prune
docs:
  - DESIGN-meshwork.md#§-9-portfolio-master-sequencing
status: open
created: 2026-08-10T16:31Z
blocked-reason:
---
Done/dropped entries accumulate: six months in, sequence.md is 200 lines
of mostly-dead ids — the clutter problem archive/ already solved for task
files, recreated in the overlay. Prune removes entries whose tasks are
done/dropped in a present repo, preserves tranche headings and their
ordering, and leaves unresolvable entries alone (an absent checkout is not
evidence of death). Rides the resolution logic the dangling lint lands
first. The surface (portfolio seq --prune, or lint --fix semantics in the
portfolio repo) is a §6 amendment — needs its own nod.

## log
- 2026-08-10T16:31Z created
- 2026-08-10T19:16Z open→blocked — awaits owner §6 nod on the prune surface — proposal in comments (recommend portfolio seq --prune); unblock and implement after the ruling
- 2026-08-10T19:21Z blocked→open

## comments
- 2026-08-10T19:16Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Surface proposal (awaits the §6 nod this body names): recommend portfolio seq --prune. (1) Prune writes portfolio state — inside the MW-A3 write boundary (repo + portfolio repo) but portfolio-repo maintenance by nature, so it belongs in the portfolio verb family: running lint --fix in repo X must never edit a file in the portfolio repo from an unrelated cwd, and satisfied entries are deliberately NOT lint findings (mw-2nmsys2 ruled them prune's business), while --fix repairs only what lint reports. (2) portfolio seq is already the ordering-maintenance verb (§15.2 renumber pending as mw-908n9k2); prune is the same family — one flag, no new verb. (3) Semantics ready to implement on the nod: remove entries resolving to done/dropped in a registered, PRESENT repo (find_task_file already reaches archive/); preserve tranche headings, prose, ordering; leave dangling and unresolvable entries alone — absent checkout is not evidence of death (MW-G5), dangling stays lint's finding. Print each removal one per line; the file is versioned, so git diff is the review surface and the undo.
- 2026-08-10T19:21Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Owner ruling 2026-08-10: no --prune flag — running any portfolio verb autoprunes. Surface question resolved: behavior on existing verbs, no flag, no new verb. Semantics per the proposal otherwise: remove entries resolving to done/dropped in a registered, present repo; preserve headings/prose/ordering; dangling and unresolvable entries never pruned; removals reported (stderr in text mode, a pruned list in JSON data); git diff in the portfolio repo is the review surface.
