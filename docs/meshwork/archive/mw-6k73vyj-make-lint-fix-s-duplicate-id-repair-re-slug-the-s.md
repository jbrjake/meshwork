---
id: mw-6k73vyj
title: "Make lint --fix's duplicate-id repair re-slug the side with fewer inbound edges, as MW-A4 requires"
category: core/hygiene
docs: [FORMAT.md#merge-semantics]
verify: run cargo test duplicate_id_fix_reslugs_fewer_inbound
status: done
created: 2026-10-02T13:39Z
---

MW-A4: `lint --fix` MUST re-slug the side with fewer inbound edges, rewriting same-repo references. The code (src/cli/lint.rs duplicate-id repair) keeps the id on the earliest `created` instead and only reports inbound refs. Reproduced: a hand copy of a task with an older `created:` kept the id and the original was re-slugged; the copy then drew a spurious `verify-changed-since-approval`, because approvals are keyed by id. Either the code follows MW-A4, or the owner amends the requirement.

## log
- 2026-10-02T13:39Z created
- 2026-10-04T16:04Z handoff by claude (5566559a-b6fd-4780-8c73-3214f8c33f5d)
- 2026-10-04T16:17Z open→doing — claimed by claude (2a6ba2e9-aa96-4775-b0f1-2a5cd137c964)
- 2026-10-04T16:28Z doing→done — verify exit 0 @ 57a3da4+2

## comments
- 2026-10-04T14:56Z [claude (70b8c3c2-a3ae-4cee-81aa-681b84b44405)] Not a quick fix: post-merge both duplicate files carry the same id, so an inbound edge names the id, never a side — FORMAT.md#merge-semantics and MW-A4 leave what a side is undefined once merged. Attributing each inbound reference to the clone it arrived with would mean reading git history per edge. Needs an owner ruling on what fewer inbound edges means after the merge, or an amendment keeping the earliest-created file as the keeper.
- 2026-10-04T16:17Z [claude (2a6ba2e9-aa96-4775-b0f1-2a5cd137c964)] Owner ruling 2026-10-04, in session: "go with the git history option" — reading 1. A side is defined by git history: a reference belongs to the side whose file was in the tree at the commit that introduced it; a reference introduced after the merge or never committed belongs to neither. The side with the most attributed references keeps the id; ties go to the side that entered history first, then the earlier created:, then the file name. The other side is re-slugged and its attributed references are rewritten.
