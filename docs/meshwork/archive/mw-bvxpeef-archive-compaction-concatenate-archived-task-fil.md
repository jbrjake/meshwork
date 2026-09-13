---
id: mw-bvxpeef
title: "Archive compaction: concatenate archived task files into multi-doc bundles"
status: done
category: core/store
verify: run cargo test e2e::archive_compact
docs:
  - docs/DESIGN-meshwork.md#§-1-on-disk-layout-per-repo
  - docs/DESIGN-meshwork.md#§-3-ingestion-pipeline
created: 2026-08-07T01:38Z
seq: 360
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
- 2026-09-13T13:52Z open→doing — claimed by claude (95c1ceaa-ef91-4ac1-90f8-d8922fd3bdd4)
- 2026-09-13T14:12Z doing→done — verify exit 0 @ c738ad9+2

## comments
- 2026-09-13T14:12Z [claude (95c1ceaa-ef91-4ac1-90f8-d8922fd3bdd4)] Landed: src/archive.rs — bundle-NNNN.md in archive/, split on a top-level --- followed by an id: line (fences respected), 512 KiB per bundle, 100 loose files before lint warns archive-loose and lint --fix compacts (a deliberate step: the first bundle makes the store format 2 and older binaries refuse it). Every verb reads/writes a task through archive::locate (Located::read/write splices a bundled document by id); reopen and the misplaced repair extract a document back out; minting collision-checks bundled ids. FORMAT.md is version 2 with a Bundles section; the conformance corpus carries a bundle with blessed rows; DESIGN §1/§3/§6/§15.14, REQUIREMENTS MW-A1, TRACE. Verify e2e::archive_compact covers threshold, cap rollover, format bump, q/ready/show/comment on bundled tasks, reopen, and below-threshold silence. NOT done here: this repo's own store now trips archive-loose (178 loose) — compacting it is the owner's call, since sibling repos pinned at v0.4.0 would refuse a format-2 store in portfolio reads until they re-pin past the next release. Gate green (verify_meshwork.sh exit 0, observed).
