---
id: mw-16pyc5g
title: "add --batch: reject unknown frontmatter keys atomically; alias from: → discovered-from:"
status: open
category: core/format
verify: cargo test e2e::batch_rejects_unknown_keys
discovered-from: mw-ntt5
seq: 25
docs:
  - DESIGN-meshwork.md#§-6-cli-surface
created: 2026-08-07T13:47Z
---
Pilot evidence (sazed): a batch document using `from:` (the slot name §6
and the --batch help text themselves use: "needs/parent/from/relates")
was accepted verbatim and produced NO edge — 6 of 13 recovered tasks
silently lost their discovered-from links; only the human user caught it
mid-session, and only lint later flagged the unknown key. The canonical
file key is `discovered-from:` (what `add --from` writes). Fix both
sides: --batch validates frontmatter keys against the schema and refuses
the whole batch on an unknown key (atomicity is already the contract),
AND `from:` is accepted as an input alias rewritten to `discovered-from:`
— the help text taught it, so the tool must honor or reject it, never
swallow it.

## log
- 2026-08-07T13:47Z created
