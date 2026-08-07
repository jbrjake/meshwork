---
id: mw-mjwfvxn
title: "Threat model + owner ruling: verify: is untrusted input"
status: done
category: core/verify
parent: mw-6895bkg
verify: grep -q MW-E5 REQUIREMENTS-meshwork.md
seq: 15
docs:
  - REQUIREMENTS-meshwork.md#§-e-lifecycle-discipline # MW-E2 sh -c today
  - REQUIREMENTS-meshwork.md#§-3-non-goals # ruling recorded here when scope moves
created: 2026-08-06
---
Deliverable is a ruling, written down: DESIGN gains a trust-boundary
section (task files that arrived via merge are untrusted input; verify:
is attacker-controlled; execution points are close + anything that runs
verifies), REQUIREMENTS gains MW-E5 (MUST: executing a verify never
grants arbitrary shell to untrusted task content) and §3 records the
scope ruling — including that a verify predicate grammar is NOT the
rejected "bespoke query language" (that fence was about querying; SQL
stays). Decide here: exact trust boundary (git-author? clone-local
approval only?), CI/test posture (a MESHWORK_TRUST env contract hook,
like MESHWORK_ID_SEED), and which §6 verbs/flags the gate may add.

## log
- 2026-08-06 created
- 2026-08-07T01:52Z open→doing — claimed by claude
- 2026-08-07T02:27Z doing→done — verify exit 0

## comments
- 2026-08-07T02:25Z [claude] Ruling recorded: DESIGN §12b (threat model: drive-by merged verify → close → RCE; execution point is close alone; mirror stays fixed-argv), REQUIREMENTS MW-E5 (MUST: TOFU per-clone approval before shell verify; MESHWORK_TRUST=1 reviewed-checkout grant; git authorship never trust), §3 scope ruling (verify DSL ≠ bespoke query language; gate adds at most close --approve), §15.11 pointer. Decisions delegated by the task body were taken as: boundary = clone operator TOFU on exact verify text (matches mw-9rc4vs6's sketch); CI posture = MESHWORK_TRUST=1 env contract joining §15.6; §6 delta = one flag, no verb. Adjacent surfaces filed: mw-2pz0zqc (path confinement), mw-8fmsws3 (terminal escapes). TRACE row MW-E5 planned → e2e::verify_trust.
