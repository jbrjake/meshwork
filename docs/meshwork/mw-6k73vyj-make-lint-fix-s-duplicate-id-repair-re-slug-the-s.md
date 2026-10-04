---
id: mw-6k73vyj
title: "Make lint --fix's duplicate-id repair re-slug the side with fewer inbound edges, as MW-A4 requires"
category: core/hygiene
docs: [FORMAT.md#merge-semantics]
verify: run cargo test duplicate_id_fix_reslugs_fewer_inbound
status: open
created: 2026-10-02T13:39Z
handoff: |
  In scope since the owner's 2026-10-04 instruction, still waiting on one
  ruling: what "the side with fewer inbound edges" means once both files
  carry the same id. Nothing in the code, FORMAT.md or MW-A4 defines a
  side post-merge, so an agent cannot pick a reading without the owner.
  
  Where the code stands: src/cli/lint.rs `fix_duplicate_ids` groups valid
  tasks by id, sorts each group by (`created`, file name) and re-slugs
  every file but the first through `reslug`; `count_inbound` counts
  same-repo references to the id after the fact and prints an advisory
  note. No reference is rewritten.
  
  Two readings, either implementable in a session:
  
  1. Attribute each inbound reference to the clone that authored it
  through git: for every referencing file, `git log -S<id> --format=%H --
  <file>` finds the commit that introduced the reference; the duplicate
  present in that commit's tree (`git cat-file -e <sha>:<path>`) is the
  side it meant. References introduced after the merge, or by an
  uncommitted edit, attribute to neither. The side with fewer attributed
  references is re-slugged, and its references are rewritten to the new
  id. An uncommitted hand copy (the reproduced case) has no history, so it
  attributes nothing and loses the id, which is what the repro wanted.
  Cost: lint --fix shells out to git, and a store outside git history
  falls back to the current rule.
  
  2. Amend MW-A4 to keep the earliest-created file (the current behaviour)
  and report inbound references instead of rewriting them. Cheapest, but
  it is what produced the spurious `verify-changed-since-approval` in the
  repro, because approvals are keyed by id and the copy with the older
  `created` kept it.
  
  The red test named in `verify:`
  (`duplicate_id_fix_reslugs_fewer_inbound`) fits reading 1: two files
  with one id, inbound edges attributable to one side by history, the
  other side re-slugged and its references rewritten. Under reading 2 the
  task closes by amending the requirement and `verify:`, which is the
  owner's edit.
---

MW-A4: `lint --fix` MUST re-slug the side with fewer inbound edges, rewriting same-repo references. The code (src/cli/lint.rs duplicate-id repair) keeps the id on the earliest `created` instead and only reports inbound refs. Reproduced: a hand copy of a task with an older `created:` kept the id and the original was re-slugged; the copy then drew a spurious `verify-changed-since-approval`, because approvals are keyed by id. Either the code follows MW-A4, or the owner amends the requirement.

## log
- 2026-10-02T13:39Z created
- 2026-10-04T16:04Z handoff by claude (5566559a-b6fd-4780-8c73-3214f8c33f5d)

## comments
- 2026-10-04T14:56Z [claude (70b8c3c2-a3ae-4cee-81aa-681b84b44405)] Not a quick fix: post-merge both duplicate files carry the same id, so an inbound edge names the id, never a side — FORMAT.md#merge-semantics and MW-A4 leave what a side is undefined once merged. Attributing each inbound reference to the clone it arrived with would mean reading git history per edge. Needs an owner ruling on what fewer inbound edges means after the merge, or an amendment keeping the earliest-created file as the keeper.
