---
id: mw-mjwfvxn
title: "Threat model + owner ruling: verify: is untrusted input"
status: open
category: core/verify
parent: mw-6895bkg
verify: grep -q MW-E5 REQUIREMENTS-meshwork.md
seq: 15
docs:
  - REQUIREMENTS-meshwork.md#§-e-lifecycle-discipline # MW-E2 sh -c today
  - REQUIREMENTS-meshwork.md#§-3-non-goals # ruling recorded here when scope moves
created: 2026-08-06
handoff: |
  Release chapter is CLOSED: v0.1.4 shipped 2026-08-07 (Actions recovered;
  GitHub flushed the original throttled webhook — see mw-0pj8qgv
  comments for the draft-release gotcha). The sazed pilot is unblocked;
  this threat-model doc is now genuinely the top of the queue, no
  pre-checks. Session 2026-08-07 landed tools you should USE here: add
  --batch (file the whole security sequence with @handles in one atomic
  call), start --as <you> (claim before working — prime/ready now
  annotate claims), and prime now shows store provenance, so trust its
  staleness line. Deliverables unchanged: DESIGN trust-boundary section,
  MW-E5, §3 scope ruling — details in the task body.
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
