---
id: mw-nq6rew9
title: Skill ritual for what changed — the write flags, the DSL tokens, tiers as bands, the asks verb
category: skill
needs: [mw-vwdm3ed, mw-ps4fzn2]
seq: 710
verify: contains .claude/skills/meshwork/SKILL.md /express tiers as bands/
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-3-recommendations
  - docs/PROPOSAL-prioritization.md#§-7-build-ladder
status: open
created: 2026-09-07T16:32Z
handoff: |
  v0.5.0 is published (2026-09-23, release run 35890107400: five assets,
  the
  notes file as the body), so the code this task describes is out. Of the
  four items, SKILL.md already carries two: the verify grammar block has
  `lacks` and `package=`/`target=`, and the authoring section lists every
  `set` flag. Still open: (1) the inbox in "The inbox and sibling stores"
  still teaches the `portfolio q … FROM asks` idiom (SKILL.md lines
  56-58);
  replace it with the `asks` verb, which lists both directions uncapped.
  (2) "express tiers as bands, keep seq for exceptions" and its migration
  note wait on the prioritization ruling (mw-ryd25rq; the owner's handoff
  on
  mw-59f0t1q says that task may be superseded) — the verify is that
  phrase,
  so this task cannot close until the bands exist by name. README.md is
  the
  reference for what shipped: every new surface has a transcript the gate
  replays (scripts/check_readme_transcripts.py stages a governor sibling
  and
  a registry for the two-sided ask).
---
One commit after the code it describes: "every field also has a CLI path" becomes true again
and says how; `package=`/`target=` and `lacks` in the grammar summary; *express tiers as bands,
keep `seq` for exceptions* with the migration note for stores that tiered by `seq`; the `asks`
verb replaces the `portfolio q` idiom in Sibling Stores. Skill rules as before.

## log
- 2026-09-07T16:32Z created
- 2026-09-23T17:10Z handoff by claude (f511b0dd-d8d9-4deb-8608-c9df0f4d5c9b)
