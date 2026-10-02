---
id: mw-eajbw04
title: "Build the notes cli's day 0 — C1–C6 and LABEL closed in its store, backlog filed with TITLE needing PFM, story/0-day0 pushed"
category: meta/demo
seq: 240
needs: [mw-z8k7fy5, mw-jbqrw7b]
verify: git -C ../meshwork-demo-notes-cli rev-parse -q --verify refs/tags/story/0-day0 && git -C ../meshwork-demo-notes-cli merge-base --is-ancestor story/0-day0 origin/main && grep -rqs 'meshwork-demo-notes-sync#docs/PROTOCOL.md#sp-change-timestamp' ../meshwork-demo-notes-cli/docs/meshwork && grep -rqs concurrent_title_edits ../meshwork-demo-notes-cli/docs/meshwork
docs:
  - docs/PLAN-demo-notes.md#repo-meshwork-demo-notes-cli
status: open
created: 2026-10-02T15:21Z
---
Work in ../meshwork-demo-notes-cli (store alias `nt`), from the corrected plan, with `MESHWORK_PORTFOLIO` at the demo portfolio.

The crate depends on notesync by git tag `v0.1.0`; commit Cargo.lock. Give it the same Rust scaffold and SHA-pinned CI as sync. `list` labels each note with `edited_ago(now, doc.timestamp_ms)`. That is the line HLC later breaks.

Day-0 tasks:
- C1–C6, each filed, built and closed by its verify.
- LABEL stays unranked and pins `meshwork-demo-notes-sync#docs/PROTOCOL.md#sp-change-timestamp`. That `cover` resolves through the demo registry, which is why this waits on the portfolio repo.
- The day-0 README task the correction adds.

Backlog:
- TITLE at seq 30 needs `meshwork-demo-notes-sync#<per-field merge id>`. Its body names per-field merge so Beat 1's `portfolio search merge` finds it.
- Export at seq 40, search at seq 50.

Tag `story/0-day0` and push. `meshwork lint` exits 0 first. Task-file commits stay store-only.

## log
- 2026-10-02T15:21Z created
