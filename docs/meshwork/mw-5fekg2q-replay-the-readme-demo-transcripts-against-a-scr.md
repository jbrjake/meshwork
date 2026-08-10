---
id: mw-5fekg2q
title: Replay the README demo transcripts against a scratch store in the gate
category: meta/readme
needs: [mw-qe5y2fc]
verify: ./scripts/check-readme-transcripts.sh
docs:
  - README.md
  - DESIGN-meshwork.md#§-14-gate
status: open
created: 2026-08-10T16:24Z
---
README line 79 promises every transcript is pasted from a real run of the
binary. Hand-pasting proves a block true only on the day it lands: when
038867f grew the demo from 2 open tasks to 4, the `q` category-rollup block
was the one transcript with no minted ids in it, so the id-regeneration
sweep that forced every other block to be re-pasted skipped it, and it
contradicted the `prime` block above it until review caught it (2026-08-10).

Automate the promise: a gate script that extracts the README's demo fences
(blocks whose first line starts `$ meshwork`), executes the commands in
order against a scratch store in a tempdir, and diffs each block's real
output against the pasted text, normalized for minted ids (`xx-xxxxxxx`),
store-prefix, timestamps, and the `store @` hash. Row order is NOT
normalized — the `q` tie-order was observed stable across 5 runs and the
pasted transcript should match it exactly. Non-meshwork lines in fences
(`touch repro.log`, `cat`, `grep`) replay as plain shell. Zero network,
like every gate section (MW-J6). Wire it into verify_meshwork.sh; failure
output names the first divergent block by README line number.

## log
- 2026-08-10T16:24Z created
