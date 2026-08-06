---
id: mw-tb6gdr9
title: Agent work claiming (beads --claim analogue)
status: open
category: core/lifecycle
verify: cargo test e2e::claim
seq: 240
docs:
  - DESIGN-meshwork.md#§-6-cli-surface # frozen verb table — amending needs the ruling
  - REQUIREMENTS-meshwork.md#§-3-non-goals # adjacent to "no assignees/roles" fence
  - REQUIREMENTS-meshwork.md#§-k-comments-attachments # MW-K1 self-professed identity
created: 2026-08-06
---
Owner-requested 2026-08-06: a way for agents to take on work, like beads'
--claim. Today `start` flips open→doing but records no actor — two
parallel worktree sessions can start the same task and merge silently
into duplicate work. Shape needs an owner ruling (frozen surface, §6):
(a) `start` grows a claimed-by write via the MW-K1 author chain;
(b) new `claim [<id>]` verb, bare form = claim top ready item (closest
to beads `ready --claim`); (c) field-only `claimed-by:` + lint. Whatever
the shape: identity stays a self-professed string (no accounts — §3
fence holds), a claim is advisory with no locking (concurrency is git's
problem; post-merge double-claim is a lint finding like duplicate IDs,
reported never auto-resolved), and `ready`/`prime` must respect claims
(exclude-or-annotate others' claims) or the field is decoration. Moving
this out of §3's default-reject list must be recorded there per its own
rule.

## log
- 2026-08-06 created
