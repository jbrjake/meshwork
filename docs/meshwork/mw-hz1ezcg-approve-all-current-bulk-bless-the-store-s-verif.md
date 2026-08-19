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
status: blocked
created: 2026-08-08T14:09Z
handoff: |
  Up next after the Gold-III lane landed whole (2026-08-17: setup-cost
  matrix -> addressed tasks -> reveal prep; see those archive files'
  comments). Weather to absorb before designing: the owner overruled
  run-stays-gated 2026-08-14 and tree-hash bless (THIS task) is the
  candidate mechanism for ever un-gating DSL run verifies — read the
  mw-hz1ezcg comment thread and mw-6895bkg's ruling first; the ride-along
  guard (mw-egksvhm) already auto-trusts store-only-provenance run
  verifies, so scope what bulk-bless still needs to add on top. One
  session-fresh trap for the e2e: `start` red-checks now print a
  'red-check skipped — verify unapproved' note (seen live in
  scripts/demo.sh output), so approval-state assertions have exact text
  to pin against. New since your docs refs: to:/answers: keys and the
  addressed join (DESIGN §15.12) — unrelated surface, but tables.rs and
  query.rs moved; rebase any stale line-number notions from the store.
blocked-reason: awaiting owner §6 ruling — bulk-bless adds a verb; REQUIREMENTS L116 caps the trust surface at close --approve (proposal in comments)
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

## comments
- 2026-08-08T16:42Z [claude] Scale check from the field (sazed, 2026-08-08): in-session refusals are few (6 across 8 sessions) because agents pre-emptively close --approve; the treadmill this task targets is per-clone re-approval — the sazed store now carries 100+ shell verifies. mw-2kgkn0j (approve-at-mint) covers the in-session half of the cost; the two compose.
- 2026-08-14T15:37Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Scope grew teeth 2026-08-14 (ruling on mw-6895bkg): tree-hash bless is now the candidate mechanism for ever un-gating DSL run predicates — approval scoped to a reviewed tree state re-arms when merged code changes what cargo would load. Until this lands, run stays gated like legacy shell (DESIGN §12b gate routing).
- 2026-08-14T15:40Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Correction to the previous comment: owner overruled run-stays-gated same day. The ride-along guard (mw-egksvhm, store-only task provenance) is what frees run verifies; tree-hash bless is the belt-and-braces escalation if the accepted split-PR residual ever bites, not the precondition.
- 2026-08-19T19:43Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Picked up as next (2026-08-19), stopped at the boundary the body has always flagged: the bulk flag is a §6 surface delta, and REQUIREMENTS L116 is a standing ruling that the trust gate 'adds no verb and at most one flag (close --approve)'. An agent can't amend a normative ruling, so this needs an owner nod. Proposed ruling, sized for a one-word yes: grow §6 by one verb, 'approve --all-current' — prints every live unapproved verify (id + full text: the on-screen review §12b requires, now once per store instead of 108 times), records this clone's content-hash approvals for all of them in one pass, and reports the count plus the docs/meshwork/ tree hash it blessed as the review anchor. No single-id spelling (close --approve already covers that); a later new-or-changed verify prompts exactly as today — the e2e (approve_bulk_then_single_prompt) pins that re-arm. Needs REQUIREMENTS L116's no-verb cap amended in the same ruling. Blocking on that; taking the next ready items meanwhile.
