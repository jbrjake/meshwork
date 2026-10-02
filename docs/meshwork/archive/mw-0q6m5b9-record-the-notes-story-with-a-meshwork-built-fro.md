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
status: done
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
- 2026-10-02T17:31Z open→doing — claimed by claude (602c381b-d7db-491e-8df6-85682e6152ed)
- 2026-10-02T17:54Z doing→done — verify exit 0 @ acc61dc+2

## comments
- 2026-10-02T17:47Z [claude (602c381b-d7db-491e-8df6-85682e6152ed)] Owner approved in session 2026-10-02 (session ): run story/replay.sh --record and push the three staged sessions (commits, story tags, notesync v0.2.0) to the public demo repos.
- 2026-10-02T17:54Z [claude (602c381b-d7db-491e-8df6-85682e6152ed)] Done: the story is recorded on meshwork 0.5.2 (story/recording.md, 2026-10-02T17:50Z). R2–R7 are closed in the demo portfolio's store. The replay runs green in all three modes: default (https clones from GitHub), --from local checkouts, and --record. It asserts every refusal, red-check, prime line, audit row and lint the beats name. GitHub CI is green on the recorded heads. Differences from the plan: Beat 3 asserts 'answered-by … (done)' on show, because v0.5.2's prime truncates that asks-out line before the gid. Code commits land before each close, so closes check a committed tree. notes-2 comments on the label when reopening it. In record mode each session starts on a new minute, so the minute-resolution log reads in order.
