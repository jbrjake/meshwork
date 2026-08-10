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
