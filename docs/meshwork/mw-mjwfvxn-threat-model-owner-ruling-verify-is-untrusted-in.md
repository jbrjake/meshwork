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
  Before starting this: check mw-0pj8qgv — if GitHub Actions has
  recovered (githubstatus.com), ship the v0.1.4 release per its
  blocked-reason (dispatch or tag re-push), close it, and the sazed pilot
  unblocks. THEN this threat-model doc is the top of the security
  sequence: deliverables are a DESIGN trust-boundary section, MW-E5, and
  the §3 scope ruling — details in the task body; the trust-gate
  stopgap (mw-9rc4vs6) is gated on it.
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
