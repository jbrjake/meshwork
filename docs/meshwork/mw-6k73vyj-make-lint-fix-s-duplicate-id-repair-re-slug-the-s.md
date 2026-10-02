---
id: mw-6k73vyj
title: "Make lint --fix's duplicate-id repair re-slug the side with fewer inbound edges, as MW-A4 requires"
category: core/hygiene
docs: [FORMAT.md#merge-semantics]
verify: run cargo test duplicate_id_fix_reslugs_fewer_inbound
status: open
created: 2026-10-02T13:39Z
---

MW-A4: `lint --fix` MUST re-slug the side with fewer inbound edges, rewriting same-repo references. The code (src/cli/lint.rs duplicate-id repair) keeps the id on the earliest `created` instead and only reports inbound refs. Reproduced: a hand copy of a task with an older `created:` kept the id and the original was re-slugged; the copy then drew a spurious `verify-changed-since-approval`, because approvals are keyed by id. Either the code follows MW-A4, or the owner amends the requirement.

## log
- 2026-10-02T13:39Z created
