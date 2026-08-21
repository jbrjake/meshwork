---
id: mw-e8hg2kt
title: close and drop strip the task's handoff block
status: open
category: core/lifecycle
verify: run cargo test strips_handoff
relates:
  - mw-4jgrjar
created: 2026-08-12T20:48Z
seq: 70
handoff: |
  Fresh context 2026-08-21: close/drop write through transition() in
  src/cli/transition.rs (~line 216) and close.rs's own write path — both
  already edit frontmatter via crate::edit. remove_scalar (src/edit.rs:52)
  now skips the indented block under a removed key (landed with the
  mw-nzeezr8 fix), so the strip is likely one remove_scalar(&text,
  "handoff") call placed where close AND drop share it; confirm reopen
  does NOT strip, and note close.rs writes its file separately from
  transition(). Sibling bug mw-gbep3j8 (set --handoff '' leaves a dangling
  'handoff: |') lives in set.rs's handoff path — same block machinery,
  fix together if natural but close separately. Red-first: e2e wants
  set-handoff → close → file free of 'handoff:'; lint's handoff-stale
  test (tests/suite/lint.rs handoff_on_done_warn) shows today's symptom.
  Session note: verify approvals for this clone keep naming arrival
  aa4453ff — it is reviewed (batch_edit.py byte-identical to code root),
  approve freely.
---
A task that closes while carrying `handoff:` leaves a lint warning
(handoff-stale) that `--fix` does not repair, on a file that has just
auto-archived — so the sanctioned fix is hand-editing an archived file,
exactly what the skill forbids. Observed end-to-end in leras (4e5b1f04
13:57: close → warning → hand-edit of docs/meshwork/archive/…), and
this store carries two live instances right now (mw-ncfg, mw-ntt5).
The voice belongs to whatever is up next; the terminal transition is
the natural place to drop it (into the log if it must survive).

## log
- 2026-08-12T20:48Z created
