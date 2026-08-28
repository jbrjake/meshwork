---
id: mw-bvxpeef
title: "Archive compaction: concatenate archived task files into multi-doc bundles"
status: open
category: core/store
verify: run cargo test e2e::archive_compact
docs:
  - docs/DESIGN-meshwork.md#§-1-on-disk-layout-per-repo
  - docs/DESIGN-meshwork.md#§-3-ingestion-pipeline
created: 2026-08-07T01:38Z
seq: 360
handoff: |
  Deploy changed 2026-08-28, session-wide: every repo's shim now lives at
  docs/meshwork/meshwork (root ./meshwork is gone here; the dev shim still
  execs target/debug/meshwork). The plugin meshwork@jbrjake serves the
  skill user-scoped; .claude/skills/meshwork/ in THIS repo is the plugin
  source, keep it. The task itself is untouched: write the red
  e2e::archive_compact test first; the archive gained five fresh migration
  closes to compact against.
---
Owner request 2026-08-06: archive/ accumulates one file per closed task
forever — DataFusion opens a file handle per task and git tracks an
ever-growing file count. Compact archived tasks by plain concatenation
(multiple §2 documents per file, the add --batch input format) into
bundles sized reasonably for git without LFS (target on the order of
256KB–1MB per bundle; pick and record the number in DESIGN §1).
Constraints: CLI surface is frozen — compaction rides an existing verb
(close's relocate step or lint --fix), never a new one. Parse must
ingest multi-doc archive files (split on document fences, filename no
longer carries the ID — by-ID lookup and id_from_filename need an
archive-aware path). reopen must split the task back OUT of its bundle
into a live file (mw-45e2qf4 relocation, inverted). Append-only history
stays intact: bundles are append-friendly under merge=union; never
rewrite live tasks. Lint checks bundle integrity.

## log
- 2026-08-07T01:38Z created
