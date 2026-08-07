---
id: mw-dg5j1sv
title: "FORMAT.md: versioned on-disk format spec + projection contract"
status: done
category: core/format
needs: [mw-zp1h12d, mw-n6nvzpa, mw-3wnhhvp]
verify: grep -qi '## projection' FORMAT.md
docs:
  - DESIGN-meshwork.md#§-2-task-file-format
  - REQUIREMENTS-meshwork.md#§-a-store
seq: 6
created: 2026-08-06
---
Owner-accepted 2026-08-06 (format-hardening review). The on-disk
contract lives inside DESIGN §2 interleaved with rationale — but the
format IS the product, so give it a normative, self-contained,
versioned spec a third party can implement a reader from without the
binary: store layout, config keys (incl. the mw-n6nvzpa marker),
frontmatter schema, tail-section grammars (incl. the mw-3wnhhvp log
grammar), ID/gid grammar, comment identity (mw-xvtf5jx), canned-verb
SQL, the table projection. MUST include a `## projection` section (the
verify greps for it) declaring the file→row projection stable and
blessing git history as the change stream: the store at commit X
projects deterministically, each commit is a delta — that section is
the adapter contract for any external reader, UI layer, or incremental
engine (.cache/tasks.jsonl stays reserved as its materialization).
Also document the history idiom (`git log -- docs/meshwork/<id>-*`)
rather than ever wrapping it in a verb.

## log
- 2026-08-06 created
- 2026-08-07T02:38Z open→doing — claimed by claude
- 2026-08-07T02:40Z doing→done — verify exit 0

## comments
- 2026-08-07T02:39Z [claude] FORMAT.md landed at repo root, 8.6KB: versioning rule (semantic bump only; minting-rule idiom for additive change; newer-than-known refuses loudly), store layout incl. archive/ + .cache reservation + exact .gitattributes/.gitignore bytes, config keys, filename + frontmatter schema (19 keys), strictness (unknown-warn, duplicate-key invalid, filename ID recovery), stamp rules, log + comment grammars (log grammar from mw-3wnhhvp verbatim), merge semantics, and the ## projection section: six tables with columns, determinism claim, git-history-as-change-stream, per-task history idiom (git log with root+archive globs — no --follow, it rejects multiple pathspecs), .cache/tasks.jsonl reserved as materialization. Comment content-hash deliberately left as the (date,author,text) tuple pending mw-xvtf5jx. Pointers added: CLAUDE.md doc map + DESIGN §2 (FORMAT.md wins on disagreement).
