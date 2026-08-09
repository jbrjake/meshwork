---
id: mw-mtn4hp8
title: "merge=union has no normative enforcement — lint MUST error when absent"
category: core/store
verify: cargo test lint::gitattributes_union_missing
docs:
  - FORMAT.md#merge-semantics
  - FORMAT.md#store-layout
status: open
created: 2026-08-09T23:17Z
---
Review finding (2026-08-09). The entire Merge semantics section is true
only if `.gitattributes` is present and correct. A clone from someone
who stripped it silently loses the union property and the section
becomes false — the failure is invisible until the first bad merge.
Writers MUST ensure the attributes file; `lint` MUST error (not warn)
when it is missing or wrong, and `lint --fix` restores it. Spec side:
promote the .gitattributes line from store-layout furniture to a
normative MUST in Merge semantics.

## log
- 2026-08-09T23:17Z created
