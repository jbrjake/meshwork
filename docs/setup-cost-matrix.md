# The setup-cost matrix

First empirical numbers on agent context-switch cost, mined from this portfolio's
own meshwork stores, session transcripts, and git history. Data through **2026-08-17 18:01Z**.
Read-only miner: `python3 scripts/mine_setup_cost.py --write` regenerates this file.

## Headlines (each number's denominator in its section below)

- An agent session reaches its first task action in a median **9.1 min**,
  carrying a median **102k tokens** of loaded context to get there
  (n=87 sessions).
- Switching repos costs nothing extra here: median ramp after a cross-repo session is
  **8.8 min** vs 11.1 min after a same-repo one (n=54/32).
  The store carries the context that the context window drops.
- A session acts on a median of **4 tasks** (p90 13, max 22);
  agent work is fan-out, not single-ticket.
- Activity is front-loaded: **70%** of all task touches land in the
  task's first 24 h, and the chance an open task ever closes decays with age
  (the decreasing-hazard premise behind deprioritizing aging tickets — see below).

## Method

- **Session** = one Claude Code transcript file (`~/.claude/projects/…/<uuid>.jsonl`);
  a conversation thread — `/clear` or a new tab starts a new one. Timestamps are the
  transcript's own. Sessions are attributed to the repo whose project directory holds them.
- **Task act** = first CLI invocation of a mutating meshwork verb
  (add, attach, block, close, comment, dep, drop, reopen, set, start); **read** = blocked, lint, next, prime, q, ready, show, tree, version, why.
- **Ramp** = minutes from the session's first timestamped event to its first task act.
- **Context at first act** = input + cache-read + cache-creation tokens on the message
  that issued it — the context the agent had loaded before it could act on a task.
- **Switch class** = the chronologically previous session (any repo): `same-repo`,
  `cross-repo`, or `cold` (>12 h since the previous session ended, or none).
- **Touch** = any post-creation `## log` row or comment on a task; store dates have
  minute resolution. Aging = time since the task's `created:`.
- Sessions ending before their repo's store existed are excluded from ramp/fan-out
  denominators (counted below as pre-adoption).

## Dataset (the denominators)

| repo | tasks | log rows | comments | sessions | post-adoption | with task act | commits | w/ session trailer | store born |
|---|---|---|---|---|---|---|---|---|---|
| sazed | 288 | 432 | 95 | 151 | 56 | 37 | 999 | 914 | 2026-08-07 |
| leras | 80 | 177 | 55 | 111 | 12 | 11 | 497 | 447 | 2026-08-10 |
| meshwork | 132 | 281 | 68 | 28 | 28 | 25 | 194 | 191 | 2026-08-05 |
| marasi | 17 | 41 | 15 | 13 | 11 | 9 | 106 | 82 | 2026-08-14 |
| tensoon | 22 | 35 | 9 | 9 | 4 | 3 | 405 | 356 | 2026-08-14 |
| oreseur | 15 | 19 | 13 | 5 | 3 | 2 | 19 | 7 | 2026-08-14 |
| **total** | **554** | **985** | **255** | **317** | **114** | **87** | **2220** | **1997** | |

## Session ramp per repo

Minutes from session start to first meshwork read / first task act; token cost carried
to the first act. `n` = post-adoption sessions with at least one task act.

| repo | n | med min→read | med min→act | p90 min→act | med ctx @act (ktok) | med out-tok before act | med min→first commit (n) |
|---|---|---|---|---|---|---|---|
| sazed | 37 | 0.3 | 18.3 | 74.4 | 138 | 51022 | 30.1 (37) |
| leras | 11 | 1.3 | 6.9 | 19.2 | 98 | 51812 | 28.2 (11) |
| meshwork | 25 | 1.5 | 8.8 | 94.5 | 59 | 20612 | 23.0 (24) |
| marasi | 9 | 1.9 | 5.0 | 8.9 | 68 | 43201 | 16.9 (9) |
| tensoon | 3 | 1.1 | 4.5 | 10.4 | 93 | 44800 | 23.6 (3) |
| oreseur | 2 | 1.8 | 6.2 | 8.2 | 79 | 35936 | 18.3 (2) |
| **all** | **87** | | **9.1** | **67.0** | **102** | | |

## Cross-repo switch cost

Ramp conditioned on what the previous session (any repo) was. This is the
`(previous-context, next-task)` pair the store keeps and everything else throws away.

| previous context | n | med min→act | p90 min→act | med ctx @act (ktok) |
|---|---|---|---|---|
| same-repo | 32 | 11.1 | 94.5 | 108 |
| cross-repo | 54 | 8.8 | 67.0 | 98 |
| cold | 1 | 65.0 | 65.0 | 304 |

## Task-touch fan-out per session

Distinct existing tasks acted on per session (IDs seen in mutating commands), plus
`add` invocations (new tasks have no prior ID). Same `n` as the ramp table.

| repo | n | med tasks/session | p90 | max | total adds | med session length (min) |
|---|---|---|---|---|---|---|
| sazed | 37 | 5.0 | 9 | 16 | 117 | 123 |
| leras | 11 | 4.0 | 9 | 21 | 19 | 75 |
| meshwork | 25 | 4.0 | 18 | 22 | 82 | 98 |
| marasi | 9 | 4.0 | 5 | 8 | 13 | 36 |
| tensoon | 3 | 4.0 | 5 | 5 | 3 | 133 |
| oreseur | 2 | 1.0 | 2 | 2 | 1 | 653 |
| **all** | **87** | **1.0** | **2** | **2** | **235** | |

## Aging vs touch

Where activity lands, by task age at the moment of the touch (848 touches on 432 tasks; 49 touches with dates before their task's `created:` — date-only stamps round to midnight — excluded):

| task age at touch | touches | share |
|---|---|---|
| < 1 h | 391 | 46% |
| 1–24 h | 207 | 24% |
| 1–3 d | 134 | 16% |
| 3–7 d | 82 | 10% |
| > 7 d | 34 | 4% |

Cycle time (created → first `done`), all repos: median 15.1 h, p90 123.9 h (n=196 closed tasks with parseable dates).

Closure hazard by age — of tasks that reached age *a* still open, how many ever
closed by data end (right-censored: tasks younger than *a* at data end excluded):

| reached age still open | n | closed later | share |
|---|---|---|---|
| 24 h | 397 | 78 | 20% |
| 3 d | 310 | 37 | 12% |
| 7 d | 187 | 11 | 6% |

## Caveats

- Transcripts exist only on this machine and are prunable; sessions here = transcripts
  present at mining time, which undercounts early work. Commit counts are repo-complete.
- Store dates have minute resolution; sub-minute ramps are floor-visible only in transcripts.
- Sessions can run in parallel tabs; the previous-session classifier picks the latest
  start, so an overlapping neighbor can class as `same-repo`/`cross-repo` arbitrarily.
- Verb detection is regex over Bash commands in transcripts; heredoc bodies and prose
  mentioning meshwork verbs can miscount. Spot-checked, not proven.
- Pre-shim sessions claimed tasks as the default author, so store-side attribution is
  partial — session attribution here comes from transcript files, not `claimed-by:`.
- The meshwork repo dogfoods the CLI: its sessions also invoke meshwork against test
  fixture stores, which inflates its `add` count and can mark a fixture act as the
  session's first act. Adopter-repo rows carry no such noise.

