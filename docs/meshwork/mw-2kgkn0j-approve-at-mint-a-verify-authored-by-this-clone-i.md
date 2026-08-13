---
id: mw-2kgkn0j
title: "Approve-at-mint: a verify authored by this clone is already trusted"
category: core/verify
relates: [mw-9rc4vs6, mw-hz1ezcg]
verify: out=$(cargo test e2e::approve_at_mint 2>&1) && echo "$out" | grep -qE 'ok\. [1-9][0-9]* passed'
seq: 290
docs:
  - DESIGN-meshwork.md#§-12b-trust-boundary
status: open
created: 2026-08-08T16:42Z
---
Field evidence (sazed, first 8 post-migration sessions, reviewed
2026-08-08): every trust refusal observed (6 of 6) was immediately
followed by `close --approve` on a verify the same session had just
authored via `add --verify` — the prompt gates the author against their
own text and has become pure ceremony. The threat model (mw-mjwfvxn) is
text arriving via merge; text minted by this clone's operator crossed no
trust boundary. Proposal: `add --verify` and `set --verify` record the
TOFU approval at write time, per clone, in the same ledger the gate
already reads (mw-9rc4vs6). Merged-in or hand-edited verify text still
prompts — that asymmetry is the point, and it needs the §12b ruling.

## log
- 2026-08-08T16:42Z created

## comments
- 2026-08-12T20:50Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Adoption-week evidence from both stores: the MW-E5 refusal fired in 5 of 5 leras post-migration sessions — every time on a verify the same session had just authored (faba7815, ea33cc32, 4545e5a6, cb94b3f3, 4e5b1f04) — and in 14 of 34 sazed sessions, ~63 --approve invocations total. Every refusal was rubber-stamped within seconds, zero outcome changes; by mid-week agents pass --approve preemptively on first attempt. The gate is training the reflex that will neutralize it against genuinely merge-arrived verifies. Approve-at-mint removes exactly the noise half (with mw-51x0wty and mw-hz1ezcg as siblings), and HEAD's set --verify content-hash re-arming composes cleanly with it.
