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

## comments
- 2026-10-04T14:56Z [claude (70b8c3c2-a3ae-4cee-81aa-681b84b44405)] Not a quick fix: post-merge both duplicate files carry the same id, so an inbound edge names the id, never a side — FORMAT.md#merge-semantics and MW-A4 leave what a side is undefined once merged. Attributing each inbound reference to the clone it arrived with would mean reading git history per edge. Needs an owner ruling on what fewer inbound edges means after the merge, or an amendment keeping the earliest-created file as the keeper.
