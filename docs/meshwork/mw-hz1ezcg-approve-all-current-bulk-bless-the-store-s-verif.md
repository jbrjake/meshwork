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
status: open
created: 2026-08-08T14:09Z
handoff: |
  The DSL migration is DONE (mw-4aqmf0t, 2026-08-14): close/start route
  through verify_dsl::classify; run executes free only on store-only
  provenance (provenance::task_provenance); cargo-test vacuity is native
  (verify_exec::require_non_vacuous, ok. N>=1 passed demanded); lint warns
  verify-shell/verify-malformed from src/lint_verify.rs. This task is now
  the live friction: nearly every pre-migration task file rode along with
  code at some point (this repo commits task+code together by ritual), so
  every DSL run verify gates once per clone — I hit it closing
  mw-4aqmf0t
  itself (refusal named 62969c2 + DESIGN-meshwork.md; --approve worked).
  approve --all-current should record approvals for every live task's
  CURRENT verify text at once, anchored to a tree hash per the 2026-08-14
  ruling on mw-6895bkg (weather); it is also the ruled escalation for the
  test-in-one-merge/task-in-a-later-one residual. Surface question for the
  owner: new verb vs a flag on close — §6 is frozen, so this needs a
  ruling either way. Trust plumbing lives in src/trust.rs (record_approval
  keyed on id+text hash); the e2e style to copy is
  e2e_verify_migration.rs (stub cargo on PATH, untrusted() helper).
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
- 2026-08-14T15:37Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Scope grew teeth 2026-08-14 (ruling on mw-6895bkg): tree-hash bless is now the candidate mechanism for ever un-gating DSL run predicates — approval scoped to a reviewed tree state re-arms when merged code changes what cargo would load. Until this lands, run stays gated like legacy shell (DESIGN §12b gate routing).
- 2026-08-14T15:40Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Correction to the previous comment: owner overruled run-stays-gated same day. The ride-along guard (mw-egksvhm, store-only task provenance) is what frees run verifies; tree-hash bless is the belt-and-braces escalation if the accepted split-PR residual ever bites, not the precondition.
