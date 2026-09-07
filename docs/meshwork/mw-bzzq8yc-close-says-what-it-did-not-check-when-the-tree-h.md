---
id: mw-bzzq8yc
title: close says what it did not check when the tree has uncommitted code and the verify is a run
category: core/lifecycle
seq: 730
verify: run cargo test close_uncommitted_notice
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-2-6-the-verify-economy
status: open
created: 2026-09-07T16:32Z
---
Two tasks were closed on a red gate through `tail <log> && close && commit`; the verify was a
`run` that passed while the gate had not. When the working tree carries uncommitted code and the
verify is a `run`, `close` prints one line naming the dirty paths it did not check. Advisory;
the close proceeds.

## log
- 2026-09-07T16:32Z created
