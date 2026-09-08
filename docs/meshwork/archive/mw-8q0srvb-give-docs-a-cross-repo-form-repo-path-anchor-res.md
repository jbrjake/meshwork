---
id: mw-8q0srvb
title: "Give docs: a cross-repo form (repo#path#anchor) resolved through the registry"
category: core/format
answers: oreseur#or-j2qxf6j
seq: 220
verify: run cargo test docs_crossrepo_ref
docs:
  - FORMAT.md#§-task-file
  - docs/PROPOSAL-spec-traceability.md#§-b-a-covers-edge
status: done
created: 2026-09-07T16:32Z
---
`needs:`/`relates:` cross repos as `repo#id`; `docs:` has no such form, so every cross-repo doc
tie is a `../<repo>/…` path that `path-escape` now refuses — 66 pointers in the portfolio store
alone, plus oreseur's two. Add `repo#path[#anchor]` to FORMAT.md's link grammar, resolve it in
`src/docs.rs` through `MESHWORK_PORTFOLIO`'s `repos.toml` (unregistered repo = reported, never
read), keep refusing bare `../`, and let `lint --fix` rewrite `../<repo>/x` to `<repo>#x` when
the registry resolves the repo. The weft's resolver and `covers:` addressing both read this one
spelling.

## log
- 2026-09-07T16:32Z created
- 2026-09-08T00:20Z open→done — verify exit 0 @ f697f65+6
