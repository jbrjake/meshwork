---
id: mw-48mzck9
title: Fix the five error messages that cost the most — foreign ids, inbox verbs, frontmatter-only flags, local addressed_to, dep add
category: core/cli
seq: 280
verify: run cargo test e2e::did_you_mean_inbox_verbs
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-2-4-discoverability-tax
  - docs/FIELD-STUDY-session-transcripts.md#§-1-3-errors
status: open
created: 2026-09-07T16:32Z
---
Five messages, each with a test: `show <foreign-id>` names the repo and prints the sibling
one-liner `(cd ../<repo> && ./docs/meshwork/meshwork show <id>)` when the registry resolves the
prefix; unknown verbs `next`/`addressed`/`inbox`/`help`/`list` point at `prime`, `ready`,
`--help` (never at `set`/`add`/`dep`); unknown `--to`/`--answers`/`--body` say
`frontmatter-only; see add --batch` instead of clap's `-- --to` tip (retired by the flags task
when R-B rules them in); a local `q` on `addressed_to = <me>` returning 0 rows prints
`note: local addressed_to is outbound; the inbox is portfolio q or prime`; `dep add A B` gets a
did-you-mean and the success line models `--needs`. No task ids in any message.

## log
- 2026-09-07T16:32Z created
