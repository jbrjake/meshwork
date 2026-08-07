---
id: mw-6wdpz1b
title: "Capture before verifiable: start gates on verify:, needs-verify stays loud"
status: open
category: core/lifecycle
verify: cargo test e2e::needs_verify
docs:
  - REQUIREMENTS-meshwork.md#§-e-lifecycle-discipline
  - DESIGN-meshwork.md#§-6-cli-surface
  - DESIGN-meshwork.md#§-7-session-integration
seq: 245
created: 2026-08-07T04:48Z
---
Owner-requested 2026-08-07. Ideas are cheaper than implementations, and
MW-E2's verify-at-filing pressure (lint warns until one is set) invites
verification tests as close to vacuous as will pass — a green `true` is
worse than an honest "not verifiable yet." Wanted: capturing work
without a verify: stays legal (it already is), but (a) it cannot START
— today `start` flips open→doing regardless; it should refuse while
verify: is empty, making "write the verify" the first unit of the work
itself — and (b) it must not rot in a backlog: ready/prime surface it
loudly as needs-verify (annotated or sectioned) so defining done-ness
stays the visible next action instead of disappearing. Decide at
implementation: whether absent-verify is plain frontmatter absence or
an explicit marker; how ready treats it (annotated vs excluded-but-
shown — MW-B6's ready SQL is normative and prime's cap is bytes);
whether lint's missing-verify warning changes once the gate makes it
structural; and --waive interplay (it shouldn't change — waive is for
the genuinely unverifiable at close, this is for the not-yet-specified
at start). Surface delta needs the §6 ruling — likely zero new verbs,
`start` just refuses, the trust gate's one-flag precedent.

## log
- 2026-08-07T04:48Z created
