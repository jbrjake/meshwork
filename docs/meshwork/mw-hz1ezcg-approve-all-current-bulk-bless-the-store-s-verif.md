---
id: mw-hz1ezcg
title: "approve --all-current: bulk-bless the store's verifies at a tree hash"
category: core/verify
parent: mw-6895bkg
relates: [mw-9rc4vs6]
verify: cargo test e2e::approve_bulk_then_single_prompt
seq: 190
docs:
  - DESIGN-meshwork.md#§-12b-trust-boundary
  - DESIGN-meshwork.md#§-6-cli-surface
status: open
created: 2026-08-08T14:09Z
---
Review finding (2026-08-08). Bless every verify in the store as of a
tree hash; afterward only new-or-changed text prompts. 108 gates ×
every fresh clone is a treadmill, and direnv proves operators will just
`MESHWORK_TRUST=1` everything to escape it — the failure mode where the
trust gate becomes worse than not having it. Extends the TOFU approval
(mw-9rc4vs6); the bulk flag is a surface delta, needs the §6 ruling.

## log
- 2026-08-08T14:09Z created

## comments
- 2026-08-08T16:42Z [claude] Scale check from the field (sazed, 2026-08-08): in-session refusals are few (6 across 8 sessions) because agents pre-emptively close --approve; the treadmill this task targets is per-clone re-approval — the sazed store now carries 100+ shell verifies. mw-2kgkn0j (approve-at-mint) covers the in-session half of the cost; the two compose.
