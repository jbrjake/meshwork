---
id: mw-8a93ap1
title: Stop lint --fix bundling from writing a second format key that bricks the store
category: core/format
seq: 5
docs: [FORMAT.md#configtoml]
verify: run cargo test bump_format_keeps_one_format_key
status: open
created: 2026-10-02T13:32Z
---
`archive::bump_format` (src/archive.rs) inserts `format = 2` after the `alias` line, then also rewrites the existing `format = 1` line to `format = 2`. A config with `alias` above an explicit `format = 1` — this repo's own `docs/meshwork/config.toml` — ends with the key twice, and every verb then exits 1 with `bad config … duplicate key format`.

Reproduced: copy this store's config.toml, .gitattributes and its 229 loose archive files into a scratch repo, run `lint --fix` (which `archive-loose` tells sessions to do), then `ready` fails. The unit test only checks `contains("format = 2")`, and e2e `archive_compact` starts from an `init` store that is already format 2, so the bump path is never exercised against a format-1 config.

## log
- 2026-10-02T13:32Z created
