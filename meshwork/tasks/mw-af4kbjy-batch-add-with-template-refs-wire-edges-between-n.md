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
atomic operation — partial failure writes nothing. Shape for the
ruling (frozen §6): likely `add --batch <file|-> ` where the input
reuses the §2 frontmatter format (which also wants mw-0f4j's fields —
batch entries need seq:/docs: too, hence relates). Include --dry-run
style preview (print would-be files, write nothing) so agents can
show the graph before committing it.

## log
- 2026-08-06 created
