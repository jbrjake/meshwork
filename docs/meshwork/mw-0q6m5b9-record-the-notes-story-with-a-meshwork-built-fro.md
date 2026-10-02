---
id: mw-0q6m5b9
title: Record the notes story on the meshwork release the demo repos pin — the demo portfolio's R2–R7 driven to story/3-resolved
category: meta/demo
seq: 245
needs: [mw-eajbw04]
verify: grep -q story/3-resolved ../meshwork-demo-notes-portfolio/story/recording.md && git -C ../meshwork-demo-notes-cli rev-parse -q --verify refs/tags/story/3-resolved && git -C ../meshwork-demo-notes-sync rev-parse -q --verify refs/tags/v0.2.0 && grep -q story/replay.sh ../meshwork-demo-notes-portfolio/README.md
docs:
  - docs/PLAN-demo-notes.md#the-replay
  - docs/PLAN-demo-notes.md#beats
status: open
created: 2026-10-02T15:21Z
---
The story's work lives in the demo portfolio's own store: R2 fixture, R3 patches, R4 session texts, R5 `story/replay.sh`, R6 record, R7 its README. Drive it there.

The recording runs the meshwork release the demo repos pin, unchanged. The demo requires no change to meshwork.

- Regenerate the fixture right before `--record`, so the report's flight sits hours before the sessions.
- Record mode pushes after each beat. A re-record force-pushes main and the story tags back to `story/0-day0`, and only before the reveal.
- The replay asserts every refusal, red-check, prime line and audit row the beats name, plus `lint` exit 0 per beat. A failed assertion is a finding to fix, never an `expect` to loosen.

Done when `story/recording.md` names `story/3-resolved`, the cli carries that tag, sync carries `v0.2.0`, and the portfolio README points at `story/replay.sh`.

## log
- 2026-10-02T15:21Z created
