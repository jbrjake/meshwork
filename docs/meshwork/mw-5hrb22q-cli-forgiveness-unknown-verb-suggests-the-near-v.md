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

## comments
- 2026-08-08T16:42Z [claude] Field evidence (sazed, 2026-08-08 review): --category rejected in 2 sessions and --doc in 1; clap's similar-argument tip recovered each, but the retry re-sends the whole command — one add took 3 attempts (--category, then --doc, then success). Alias both.
- 2026-08-09T23:35Z [Jon Rubin] Field evidence (sazed, 2026-08-09, session 2436461e): --doc rejected again — two adds chained in one command, both failed, retried whole. Compounding detail: the agent's habitual '| tail -3' cut clap's 'unexpected argument' line, so the visible output was bare usage with no tip at all. Agents truncate output routinely; whatever the error must teach has to survive truncation from either end.
