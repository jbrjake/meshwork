---
id: mw-zwa403g
title: Measure what the owner costs the agents — add stop_reason waits to mine_sessions.py and re-cut
category: analysis
seq: 240
verify: contains docs/FIELD-STUDY-session-transcripts.md /owner-active minutes/
docs:
  - docs/ASKS-analytics-and-field-study.md#§-1-the-measurement
  - docs/ASKS-analytics-and-field-study.md#§-7-reproduction
status: open
created: 2026-09-07T16:32Z
---
The study's scoring is blind to a session that finished its turn and sat: no prompt, no call, no
error. Add `stop_reason` to the event stream in `scripts/mine_sessions.py`, implement the ASKS
§7 wait algorithm (main chain only, stop at the next assistant record, `isMeta` excluded,
queued prompts as zero waits), emit the wait table (count, p50/p75/p90/p99, total idle, share in
waits > 1 h, live-elsewhere and owner-active minutes) and turn-ends with no following prompt.
Land the table and the session-state primitive as a new §1 subsection of the field study.

## log
- 2026-09-07T16:32Z created
