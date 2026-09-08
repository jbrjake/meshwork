---
id: mw-tkgvsdz
title: Validate edge targets and docs anchors at add, and warn on shell metacharacters in an inline body
category: core/authoring
seq: 300
verify: run cargo test add_refuses_dangling_edge
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-2-5-defects
status: done
created: 2026-09-07T16:32Z
---
`add --from mw-mjwfxn` (typo) minted two dangling edges silently. Refuse a same-repo
`--from`/`--needs`/`--parent` target that does not exist; warn on a cross-repo target the
registry cannot resolve; warn `anchor-missing` for `--docs` at add rather than at the next lint;
warn when `--body` inline text contains a backtick or `$(` and point at `@file`/`-`. The flags
task reuses this validation for `--to`/`--answers`/`--relates`.

## log
- 2026-09-07T16:32Z created
- 2026-09-08T00:44Z open→done — verify exit 0 @ 86f462b+9
