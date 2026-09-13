---
id: mw-8aw53z9
title: Skill text the transcripts keep asking for — the four sentences, the inbox section, the DSL grammar, the authoring lead, the portfolio install ritual
category: skill
seq: 340
verify: contains .claude/skills/meshwork/SKILL.md /a store comment is not a ruling/
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-3-recommendations
  - docs/FIELD-STUDY-session-transcripts.md#§-2-7-behavioral-psychology
status: done
created: 2026-09-07T16:32Z
---
One commit, prose only, describing what exists today. The four sentences: a spoken instruction
becomes a task before the work starts, and the answer to "file it" is the id; never attribute a
request or ruling to the owner unless it is in this session's transcript — a store comment is
not a ruling; owner-scoped fields surface as a conflict, never resolved by editing the task;
memory files are yours, the store is the team's. Rewrite Sibling Stores around the inbox with
the verbatim `portfolio q` query and the sibling one-liner; the ten-line DSL grammar and that
`run` rejects flag tokens; `q --json`'s `data.rows`; lead authoring with `--body/--docs/--seq`
and `--batch --dry-run`; when hand-editing is not legal; `MESHWORK_ID_SEED`/`MESHWORK_TODAY`;
the portfolio store's shim + hook install in `references/install.md`. Skill rules: SKILL.md
stays daily-use; setup goes in the references.

## log
- 2026-09-07T16:32Z created
- 2026-09-13T13:43Z open→doing — claimed by claude (95c1ceaa-ef91-4ac1-90f8-d8922fd3bdd4)
- 2026-09-13T13:52Z doing→done — verify exit 0 @ e75828c+3

## comments
- 2026-09-13T13:52Z [claude (95c1ceaa-ef91-4ac1-90f8-d8922fd3bdd4)] Landed as one prose commit: SKILL.md rewritten at 6.9KB of the 8KB budget — the four rules up top, the inbox section around the asks view with the verbatim portfolio q, the sibling one-liner and the to:/answers:/relates: batch document, the authoring lead (--body/--docs/--seq, --batch --dry-run, handles), when hand-editing is not legal, MESHWORK_ID_SEED/MESHWORK_TODAY, the DSL grammar with the dash-token refusal, q --json data.rows; the portfolio store's pin + shim + hook + repos.toml ritual in references/install.md. Every claim checked against the binary's --help and a live portfolio q. Gate green (verify_meshwork.sh exit 0, observed).
