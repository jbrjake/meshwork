---
id: mw-z8k7fy5
title: "Build notesync's day 0 — S1–S8 closed in its store as built, backlog filed, v0.1.0 and story/0-day0 pushed"
category: meta/demo
seq: 235
needs: [mw-y5zmkxb, mw-wyvgkt0]
verify: git -C ../meshwork-demo-notes-sync rev-parse -q --verify refs/tags/v0.1.0 && git -C ../meshwork-demo-notes-sync rev-parse -q --verify refs/tags/story/0-day0 && git -C ../meshwork-demo-notes-sync merge-base --is-ancestor story/0-day0 origin/main && grep -rqs per_field_merge ../meshwork-demo-notes-sync/docs/meshwork
docs:
  - docs/PLAN-demo-notes.md#repo-meshwork-demo-notes-sync
status: open
created: 2026-10-02T15:21Z
---
Work in ../meshwork-demo-notes-sync (store alias `sy`), from the corrected plan.

For each of S1–S8 in order: file the task, build its work, and close it with `meshwork close` (verify exit 0). Then file the day-0 README task the correction adds (staged disclosure and the replay pointer).

The crate:
- Std only, toolchain 1.97.0.
- The Rust gate scaffold: `[profile.dev]`, lint flags in `.cargo/config.toml`, one test target at `tests/notesync/main.rs`.
- CI with actions pinned by commit SHA.
- `docs/PROTOCOL.md` clause text exactly as the plan gives it, because the pins hash it.

Then file the open backlog: per-field merge at seq 10 and log compaction at seq 20, each with its verify. Tag `v0.1.0` at S8, and `story/0-day0` once the backlog is filed. Push main and both tags; the cli fetches `v0.1.0` from GitHub.

Commits follow Conventional Commits, and task-file commits stay store-only. `meshwork lint` exits 0 there before the tag.

## log
- 2026-10-02T15:21Z created
