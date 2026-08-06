---
id: mw-af4kbjy
title: "Batch add with template refs: wire edges between not-yet-minted tasks"
status: open
category: core/authoring
verify: cargo test e2e::add_batch
seq: 250
relates: [mw-0f4j]
docs:
  - DESIGN-meshwork.md#§-6-cli-surface # frozen verb table — needs the ruling
  - DESIGN-meshwork.md#§-2-task-file-format # batch input should reuse this
created: 2026-08-06
---
Owner-requested 2026-08-06, from observed friction: filing the six-task
verify-security sequence (mw-6895bkg) required a wrapper shell script
capturing each minted id into variables just to wire --parent/--needs —
sibling tasks can't reference each other before they exist, so the
orchestration happened outside the tool. Wanted: one batch input
declaring several tasks with local symbolic handles (e.g. @parent,
@grammar) usable anywhere an id is (needs, parent, from, relates);
meshwork mints real ids, rewrites the refs, writes all files in one
atomic operation — partial failure writes nothing. FORMAT RULED
(owner delegated the choice 2026-08-06; proposal accepted): input is
concatenated §2 task documents — the exact on-disk format agents
already read and write, no new syntax to learn. Each block is normal
frontmatter (+optional body) with `id:` omitted and a new local-only
`handle: <name>` key; `@<name>` is legal anywhere an id is (needs,
parent, from, relates) and handles never persist to disk. Surface:
`add --batch <file|->` (stdin is the agent-natural path) +
`--dry-run` printing the would-be files, writing nothing. Batch
entries accept every §2 field, so this lands after mw-0f4j.

## log
- 2026-08-06 created
