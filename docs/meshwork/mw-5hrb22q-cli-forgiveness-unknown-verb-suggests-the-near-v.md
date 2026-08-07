---
id: mw-5hrb22q
title: "CLI forgiveness: unknown verb suggests the near verb (log→comment); --category aliases --cat"
status: open
category: core/lifecycle
verify: cargo test e2e::cli_forgiveness
discovered-from: mw-ntt5
seq: 265
docs:
  - DESIGN-meshwork.md#§-6-cli-surface
created: 2026-08-07T13:47Z
---
Pilot evidence (sazed work session): the agent ran `log <id> "harness
landed + red-watched; SF1 cold run in flight"` — a natural guess, task
files HAVE a log: section — got only "For more information, try
'--help'.", never retried with `comment`, and the in-flight progress
note was silently lost (zero comments landed all session). Separately
`add --category engine/scale` was rejected (only --cat exists), costing
a round-trip on the session's most important finding. Wanted: unknown
verbs/flags fail with a did-you-mean naming the near miss (log →
comment, --category → --cat); error text must carry the reason, not
just the usage pointer. Suggestions in error text are not a surface
change; any actual alias rides the §6 ruling on [[set-fields]].

## log
- 2026-08-07T13:47Z created
