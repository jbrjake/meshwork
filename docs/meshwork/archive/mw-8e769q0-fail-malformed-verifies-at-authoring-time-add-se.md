---
id: mw-8e769q0
title: Fail malformed verifies at authoring time — add/set --verify refuse, start refuses, add --help stops saying sh -c
category: core/verify
seq: 330
verify: run cargo test add_refuses_malformed_dsl
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-2-5-defects
  - docs/DESIGN-meshwork.md#§-12b-trust-boundary
status: done
created: 2026-09-07T16:32Z
---
`add --verify 'run cargo test -p leras topn'` mints; only `verify`/`close` refuse. Classify at
`add --verify`, `set --verify` and `add --batch` and refuse `Malformed` with the grammar line in
the message; make `start` refuse a verify `close` will refuse instead of warning and
transitioning; rewrite `add --help`'s `--verify` line (it still describes `sh -c`); put the
ten-line DSL grammar in `verify --help` and `close --help`.

## log
- 2026-09-07T16:32Z created
- 2026-09-08T00:32Z open→done — verify exit 0 @ 1b8cda5+14
