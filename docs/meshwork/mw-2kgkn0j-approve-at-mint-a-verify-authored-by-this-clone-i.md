---
id: mw-2kgkn0j
title: "Approve-at-mint: a verify authored by this clone is already trusted"
category: core/verify
relates: [mw-9rc4vs6, mw-hz1ezcg]
verify: cargo test e2e::approve_at_mint
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
