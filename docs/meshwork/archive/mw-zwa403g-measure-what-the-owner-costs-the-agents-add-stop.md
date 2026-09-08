---
id: mw-zwa403g
title: Measure what the owner costs the agents — add stop_reason waits to mine_sessions.py and re-cut
category: analysis
seq: 240
verify: contains docs/FIELD-STUDY-session-transcripts.md /owner-active minutes/
docs:
  - docs/ASKS-analytics-and-field-study.md#§-1-the-measurement
  - docs/ASKS-analytics-and-field-study.md#§-7-reproduction
status: done
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
- 2026-09-08T13:39Z open→doing — claimed by claude (8c25d21d-f70c-4d87-a136-dac674b1ca92)
- 2026-09-08T13:51Z doing→done — verify exit 0 @ 5151458+3

## comments
- 2026-09-08T13:51Z [claude (8c25d21d-f70c-4d87-a136-dac674b1ca92)] Landed as field study §1.6 (re-cut 2026-09-08, 484 transcripts, 40 d): 1,476 end_turn messages — 731 answered by a prompt (260 queued, zero wait), 314 the agent went past on its own, 431 with no prompt after. Total idle 545 h, 91% in waits > 1 h, four multi-day resumptions hold 287 h. The ask's 153 h 'owner-active' reproduces as 152 h but measures any record elsewhere (other agents included); the owner-attending number is 81 h (≥ 15 min waits) / 76 h (≥ 1 h), 16 of 41 hour-plus waits holding ≥ 30 min of it. The ask's wait counts were per record, not per message. The session-state primitive holds: 68,822 tool_use blocks, zero orphans. Gate green (verify_meshwork.sh exit 0, observed).
