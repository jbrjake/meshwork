---
id: mw-nq6rew9
title: Skill ritual for what changed — the write flags, the DSL tokens, tiers as bands, the asks verb
category: skill
needs: [mw-vwdm3ed, mw-ps4fzn2]
seq: 710
verify: run cargo test package=meshwork target=suite arch::skill_names_every_verb_and_flag
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-3-recommendations
  - docs/PROPOSAL-prioritization.md#§-7-build-ladder
status: done
created: 2026-09-07T16:32Z
---
One commit after the code it describes: "every field also has a CLI path" becomes true again
and says how; `package=`/`target=` and `lacks` in the grammar summary; *express tiers as bands,
keep `seq` for exceptions* with the migration note for stores that tiered by `seq`; the `asks`
verb replaces the `portfolio q` idiom in Sibling Stores. Skill rules as before.

## log
- 2026-09-07T16:32Z created
- 2026-09-23T17:10Z handoff by claude (f511b0dd-d8d9-4deb-8608-c9df0f4d5c9b)
- 2026-09-25T14:06Z open→doing — claimed by claude (192e60bd-9013-4a32-8fec-f25f918881a9)
- 2026-09-25T14:18Z doing→done — verify exit 0 @ 90acfe3+3

## comments
- 2026-09-25T14:06Z [claude (192e60bd-9013-4a32-8fec-f25f918881a9)] Re-verified. The old verify pinned the phrase 'express tiers as bands' from the prioritization proposal; the store records that item as rejected (mw-ryd25rq comment 2026-09-20, commit 9cb8424), so the task could never close and the seven v0.5.0 verbs it never listed (asks, stats, cover, spec, portfolio search/stats/spec) shipped untaught. The new verify is the arch test that fails whenever SKILL.md misses a verb, sub-verb, or add/set flag — observed red at this commit with 17 surfaces missing.
- 2026-09-25T14:18Z [claude (192e60bd-9013-4a32-8fec-f25f918881a9)] Closed after a green nine-section gate (this session). Sineya audit applied to SKILL.md: RIGID declared, the four rules first and the done gate last, six observed rationalizations named from the field study, seven negatives flipped positive, verify shapes moved to references/verifies.md and the spec ritual to references/spec.md; 8014 of 8192 bytes.
