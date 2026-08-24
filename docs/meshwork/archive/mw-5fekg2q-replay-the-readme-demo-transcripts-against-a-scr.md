---
id: mw-5fekg2q
title: Replay the README demo transcripts against a scratch store in the gate
category: meta/readme
needs: [mw-qe5y2fc, mw-4r7v8vj]
verify: ./scripts/check-readme-transcripts.sh
docs:
  - README.md
  - DESIGN-meshwork.md#§-14-gate
status: done
created: 2026-08-10T16:24Z
seq: 290
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
- 2026-08-24T18:11Z open→doing — claimed by claude (session_016iEafFdzwyKAtsU3AEMhaU)
- 2026-08-24T20:06Z doing→done — verify exit 0 @ e361089+2

## comments
- 2026-08-23T20:25Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Scoping this found approve-at-mint invalidated the README's trust-gate transcripts (filed mw-4r7v8vj, now a hard dep). Everything else about the script is specified in the body and stands: extract fences whose first line starts '$ meshwork', execute in order against a scratch store (the quick-start needs a cargo project with a passing stuff::thing test — see how the close block's output embeds cargo test lines), diff normalized for ids/timestamps/store-hash, elision lines ('...' and the curated cargo output) argue for in-order subsequence matching rather than strict equality.
- 2026-08-24T13:54Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Staging correction from mw-4r7v8vj's real runs (2026-08-24): the scratch cargo project must stage stuff::thing RED at transcript start and fix it inside the quick-start's '...' elision — start red-checks verifies now, and a green-at-start verify adds a warning line the README's start block doesn't show. Post-re-voice, close blocks carry no --approve preambles. Raw probe captures live on mw-4r7v8vj's attachment.
- 2026-08-24T20:06Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Landed as gate §9 (e361089): scripts/check-readme-transcripts.sh → check_readme_transcripts.py. Two deliberate deltas from the filed spec: (1) selector is every $-led fence, not just $ meshwork — the grep handle-rewrite block replays too; install/getting-it/task-file fences still excluded (no $ prompt). (2) determinism addition: ready orders by (seq, created) with no id tiebreaker, so same-minute adds left tied rows to engine luck — each replayed command gets a distinct MESHWORK_TODAY minute, pinning add order. Red-proven on three README mutations (changed text, swapped q rows, wrong author), 6/6 deterministic clean runs; staging per mw-4r7v8vj probes (red stuff::thing fixed at the elision, alias sa hand-set, commit before prime).
