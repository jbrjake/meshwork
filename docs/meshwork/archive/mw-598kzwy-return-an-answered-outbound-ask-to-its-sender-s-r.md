---
id: mw-598kzwy
title: "Return an answered outbound ask to its sender's ready and next, led by its handoff"
category: capability/asks
seq: 215
needs: [mw-arz9tdz]
verify: run cargo test answered_ask_returns_to_sender
docs:
  - docs/PLAN-demo-notes.md#meshwork-prerequisites
  - FORMAT.md#asks--obligations-across-the-boundary
status: dropped
created: 2026-10-02T15:21Z
---
Implements the P1 ruling as ruled; build nothing before it.

As proposed: an outbound ask whose answer is done leaves `asks out` and joins this repo's `ready` with `answered-by <gid> (done)`, and prime's next block can lead with it, its `handoff:` included. An ask whose answer is live stays under `asks out`. The frozen `ready` SQL (DESIGN §5) changes, so its golden is re-blessed with a reviewed diff.

The skill's inbox section teaches the asker's half: own asks leave `ready` for `asks out`, and you close the ask once its answer is done. Its text changes in the same commit.

## log
- 2026-10-02T15:21Z created
- 2026-10-02T17:02Z open→dropped — The demo runs on meshwork as released and requires no change to meshwork; P1 existed only to put the answered ask in the next slot of a scripted digest, and v0.5.2 prime already lists it under asks out with answered-by (done). Owner, in session 2026-10-02: "I NEVER EVER EVER WOULD HAVE APPROVED MAKING MESHWORK CHANGES AS PART OF THIS."
