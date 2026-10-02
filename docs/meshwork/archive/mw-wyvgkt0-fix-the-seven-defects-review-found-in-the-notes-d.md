---
id: mw-wyvgkt0
title: Fix the seven defects review found in the notes-demo plan before anything is built
category: meta/demo
seq: 200
verify: "all(contains docs/PLAN-demo-notes.md /S4.*--repin/, contains docs/PLAN-demo-notes.md /tests\\/notes\\/fixtures\\/gate-rewrite/, contains docs/PLAN-demo-notes.md /placeholder/, contains docs/PLAN-demo-notes.md /S8 .*CHANGELOG/, contains docs/PLAN-demo-notes.md /\\*\\*cli repo\\*\\*.*R1/)"
relates: [mw-5xpt8ak]
docs:
  - docs/PLAN-demo-notes.md#beats
  - docs/PLAN-demo-notes.md#work-breakdown
  - docs/meshwork/attachments/mw-wyvgkt0/plan-review.md
status: done
created: 2026-10-02T15:21Z
attachments: [attachments/mw-wyvgkt0/plan-review.md]
---

docs/PLAN-demo-notes.md was checked against the source before any demo repo exists. Every build task reads the plan, so fix it first. It is still untracked and lands in one doc commit once corrected.

The attached plan-review.md gives each defect with its evidence and fix:
1. Beat 3's audit gets two re-open candidates, because S4's pin on the rewritten conflict clause drifts too. sync-1 repins S4 in Beat 2.
2. The re-enactment reads untracked report logs, so cli main goes red once its ignore drops. Commit the logs as a test fixture.
3. Patches cannot carry runtime ids. Use placeholders.
4. Demo-mode clones already hold v0.2.0 and the story tags. Reset main to `story/0-day0` and drop the later tags.
5. S8's verify is green from S1. Check the CHANGELOG instead.
6. The cli's day 0 resolves LABEL's pin and TITLE's need through the demo registry. The cli depends on R1, and builders export `MESHWORK_PORTFOLIO`.
7. A re-record wipes post-story READMEs in cli and sync. Their disclosure lands at day 0.

It also lists the smaller fixes and the plan's claims that checked out.

## log
- 2026-10-02T15:21Z created
- 2026-10-02T16:07Z open→doing — claimed by claude (3bd8766d-768f-4aa2-99a9-538521748de6)
- 2026-10-02T16:11Z doing→done — verify exit 0 @ 4936ff6+3
