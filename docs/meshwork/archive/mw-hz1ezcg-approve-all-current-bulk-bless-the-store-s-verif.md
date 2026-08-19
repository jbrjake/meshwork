---
id: mw-hz1ezcg
title: "approve --all-current: bulk-bless the store's verifies at a tree hash"
category: core/verify
parent: mw-6895bkg
relates: [mw-9rc4vs6]
verify: run cargo test e2e::approve_bulk_then_single_prompt
seq: 190
docs:
  - DESIGN-meshwork.md#§-12b-trust-boundary
  - DESIGN-meshwork.md#§-6-cli-surface
status: dropped
created: 2026-08-08T14:09Z
blocked-reason:
---
Review finding (2026-08-08). Bless every verify in the store as of a
tree hash; afterward only new-or-changed text prompts. 108 gates ×
every fresh clone is a treadmill, and direnv proves operators will just
`MESHWORK_TRUST=1` everything to escape it — the failure mode where the
trust gate becomes worse than not having it. Extends the TOFU approval
(mw-9rc4vs6); the bulk flag is a surface delta, needs the §6 ruling.

## log
- 2026-08-08T14:09Z created
- 2026-08-19T19:46Z open→blocked — awaiting owner §6 ruling — bulk-bless adds a verb; REQUIREMENTS L116 caps the trust surface at close --approve (proposal in comments)
- 2026-08-19T21:47Z blocked→dropped

## comments
- 2026-08-08T16:42Z [claude] Scale check from the field (sazed, 2026-08-08): in-session refusals are few (6 across 8 sessions) because agents pre-emptively close --approve; the treadmill this task targets is per-clone re-approval — the sazed store now carries 100+ shell verifies. mw-2kgkn0j (approve-at-mint) covers the in-session half of the cost; the two compose.
- 2026-08-14T15:37Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Scope grew teeth 2026-08-14 (ruling on mw-6895bkg): tree-hash bless is now the candidate mechanism for ever un-gating DSL run predicates — approval scoped to a reviewed tree state re-arms when merged code changes what cargo would load. Until this lands, run stays gated like legacy shell (DESIGN §12b gate routing).
- 2026-08-14T15:40Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Correction to the previous comment: owner overruled run-stays-gated same day. The ride-along guard (mw-egksvhm, store-only task provenance) is what frees run verifies; tree-hash bless is the belt-and-braces escalation if the accepted split-PR residual ever bites, not the precondition.
- 2026-08-19T19:43Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Picked up as next (2026-08-19), stopped at the boundary the body has always flagged: the bulk flag is a §6 surface delta, and REQUIREMENTS L116 is a standing ruling that the trust gate 'adds no verb and at most one flag (close --approve)'. An agent can't amend a normative ruling, so this needs an owner nod. Proposed ruling, sized for a one-word yes: grow §6 by one verb, 'approve --all-current' — prints every live unapproved verify (id + full text: the on-screen review §12b requires, now once per store instead of 108 times), records this clone's content-hash approvals for all of them in one pass, and reports the count plus the docs/meshwork/ tree hash it blessed as the review anchor. No single-id spelling (close --approve already covers that); a later new-or-changed verify prompts exactly as today — the e2e (approve_bulk_then_single_prompt) pins that re-arm. Needs REQUIREMENTS L116's no-verb cap amended in the same ruling. Blocking on that; taking the next ready items meanwhile.
- 2026-08-19T21:47Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Owner ruling (2026-08-19): dropped. Approval must not be shotgunned — a bulk-bless of every verify at a tree hash is a security hole waiting to happen. Resolves the §6 question this task was blocked on: no bulk verb.
