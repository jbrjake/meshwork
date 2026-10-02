---
id: mw-eajbw04
title: "Build the notes cli's day 0 — C1–C6 and LABEL closed in its store, backlog filed with TITLE needing PFM, story/0-day0 pushed"
category: meta/demo
seq: 240
needs: [mw-z8k7fy5, mw-jbqrw7b]
verify: git -C ../meshwork-demo-notes-cli rev-parse -q --verify refs/tags/story/0-day0 && git -C ../meshwork-demo-notes-cli merge-base --is-ancestor story/0-day0 origin/main && grep -rqs 'meshwork-demo-notes-sync#docs/PROTOCOL.md#sp-change-timestamp' ../meshwork-demo-notes-cli/docs/meshwork && grep -rqs concurrent_title_edits ../meshwork-demo-notes-cli/docs/meshwork
docs:
  - docs/PLAN-demo-notes.md#repo-meshwork-demo-notes-cli
status: done
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
- 2026-10-02T17:22Z open→doing — claimed by claude (602c381b-d7db-491e-8df6-85682e6152ed)
- 2026-10-02T17:28Z doing→done — verify exit 0 @ 1c77aeb+2

## comments
- 2026-10-02T17:28Z [claude (602c381b-d7db-491e-8df6-85682e6152ed)] Done: the notes cli's day 0 is built in its own store. C1–C4, LABEL, C6 and C7 were each filed, built red-first and closed on its verify (nt-vqcd4a8 … nt-4gtpn48). LABEL (nt-emb4j6k) is unranked and covers meshwork-demo-notes-sync#docs/PROTOCOL.md#sp-change-timestamp through the demo registry. Backlog: TITLE nt-x765gh3 (seq 30, needs meshwork-demo-notes-sync#sy-cycv60g), export nt-tk5d5es (40), search nt-0wsk28h (50). portfolio search merge finds TITLE and PFM; why on TITLE names sy-cycv60g (open). story/0-day0 (0e9d690) is pushed; GitHub CI is green (11 tests). TITLE's title differs from the plan's: per-field merge cannot keep two renames of one field, so it reads 'Keep a rename when another device edits the note's body at the same time'. The verify is unchanged and the plan row is updated to match.
