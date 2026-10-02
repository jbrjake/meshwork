---
id: mw-arz9tdz
title: Rule on P1 — an answered outbound ask returns to its sender's ready and next
category: meta/ruling
seq: 205
docs:
  - docs/PLAN-demo-notes.md#meshwork-prerequisites
status: open
created: 2026-10-02T15:21Z
verify: contains docs/meshwork/mw-arz9tdz-rule-on-p1-an-answered-outbound-ask-returns-to-i.md /^- 2026-\d\d-\d\d owner ruled P1/
---
The notes demo's Beat 3 opens on a prime whose next block is the answered ask, led by the previous session's handoff. mw-pcjm4pb took outbound asks out of the sender's ready and next for good. Once the answer is done, the asker's prime leads with unrelated backlog, and a handoff on the ask stays hidden until someone closes the ask and re-runs prime.

**Proposal (the plan's P1).** An ask whose answer is done is the asker's next action: verify in your own tree, then close. `ready` lists it with `answered-by … (done)`, and prime's next block can lead with it, handoff included. An ask whose answer is still live stays under `asks out`.

**If rejected,** the plan's fallback stands: notes-1 leaves the case open and hands off on the case, and notes-2 closes the ask first, then re-runs prime. Drop the implementation task then and `dep rm` the recording's need on it.

The owner records the ruling in this file as its own line, `- 2026-MM-DD owner ruled P1: accepted` or `rejected`, followed by the owner's words. An agent never writes that line.

## log
- 2026-10-02T15:21Z created
