# ASKS — from the first reader, to the analytics and field-study lanes

**Status: requests, unruled. Nothing filed, no task minted, no store touched.** Written 2026-09-07
by the session that wrote `portfolio/PROPOSAL-meshwork-ui.md` — meshwork's first prospective
third-party reader — after reading `PROPOSAL-analytics.md`, `PROPOSAL-prioritization.md` and
`FIELD-STUDY-session-transcripts.md` whole, and then trying to write a design that consumes all
three. Everything below is either a hole I hit while writing against those specs or a
measurement I took today that neither lane has. §6 separates what was observed from what is
argued; §7 is the reproduction.

**The one-paragraph version.** Two of these are worth doing before anything currently on either
ladder. The first is a **one-flag change to `clock`** that turns the whole thirteen-view layer
into a time machine for free. The second is that **the analytics conformance corpus cannot
currently test the third of the view layer that only the union can produce** — a second
implementation that got every cross-repo column wrong would pass. Then: the field study measured
what agents cost the owner in friction and never measured **what the owner costs the agents in
latency**, which is 588 agent-hours over 44 days and the largest quantified number anywhere in
this body of work — and its own scoring function is structurally blind to it. The rest are
smaller: pin the closure convention before a third implementation forks it, price the findings
with data already in the transcripts, and agree one event envelope so the two event streams can
sit on one timeline.

---

## 1. The measurement neither lane has: what the owner costs the agents

The field study ranks its findings "by what they cost the owner." The inverse was never
measured, and it is larger than anything in the study.

**Method.** Every transcript on this machine — 569 files, all projects, not only the nine
registered repos — scanned 2026-09-07. A *wait* is the interval between an assistant record
carrying `stop_reason: "end_turn"` and the next real human prompt on the same main chain.
Sidechains, `isMeta` records and tool results are excluded; a turn the agent continued on its own
is not a wait. "Live elsewhere at the stop" means another transcript on this machine received a
record within five minutes of the agent stopping. "Owner-active minutes inside" counts distinct
minutes within the wait window carrying a record in a *different* transcript.

| measure | value | denominator / note |
|---|---|---|
| corpus | **569 transcripts, 2026-07-26 → 09-07 (44 d)** | all projects on the machine; the field study's 449 is the nine repos + code root |
| measured waits | **1,233** | in 312 transcripts |
| turn-ends with **no** following prompt | **1,594** | the session simply ended there — see the floor argument below |
| median wait | **1.7 min** | the median says nothing about the problem |
| p75 / p90 / p99 | 5.3 min / **15.3 min** / 11.6 h | |
| **total agent idle waiting on the owner** | **588 h** | over 44 days |
| share of that idle in waits > 1 h | **86%** | it is not a drip; it is a tail |
| waits ≥ 15 min | **125**, 540 h idle | of which **106 (85%) began while another session on this machine was live** |
| waits ≥ 1 h | **40**, 504 h idle | of which **37 (92%) began while another session was live** |
| **owner-active minutes falling inside those waits** | **153 h** (≥ 15 min waits) · **138 h** (≥ 1 h waits) | minutes in which the owner was demonstrably at the keyboard, in another repo, while this agent sat |
| median owner-active minutes per long wait | 14 min (≥ 15 min) · **56 min** (≥ 1 h) | |
| long waits containing ≥ 30 min of owner activity | 31 of 125 (25%) · **26 of 40 (65%)** for the hour-plus ones | |
| distinct days carrying a wait ≥ 15 min | **34 of 44** | not an incident; the working pattern |

**Read the 588 and the 153 differently.** 588 h is agent *latency*, not spend — an idle session
burns no tokens, and much of that tail is night and away-from-desk. The number that is not
explainable that way is **153 h**: minutes where the machine can prove the owner was working,
in another session, while this one held a finished turn. That is the recoverable part, and it is
still enormous. The 65% figure is the sharpest single line here: of the forty waits over an hour,
twenty-six contained half an hour or more of demonstrated owner activity elsewhere.

**588 h is a floor, not a total.** 1,594 turn-ends had no following prompt at all — the session
ended there. An unknown fraction of those are the terminal form of the same defect: the owner
never came back, and the work in flight was abandoned rather than resumed. Separating "closed
deliberately" from "walked away" needs a signal I did not find in the records; if the field-study
lane can find one, the real number is larger and possibly much larger.

### 1.1 Why the study's own scoring cannot see this

§5's weights, verbatim: prompts citing meshwork or a task id (×4, ×6 with correction language),
emphatic prompts followed by a mutating verb (×5), emphatic prompts naming meshwork (×4),
log-scaled call and mutation counts, errors (×2), guessed verbs, denied calls (×5), hand-edits
(×2.5), `--help` probes, skill loads.

**Every component requires either the owner to type something or the agent to call something.**
A session that finished its turn and sat for four hours produces neither: no prompt, no call, no
error, no edit, no probe. It scores approximately zero and is invisible to the close-reading
selection that the whole study's qualitative half rests on. The corpus was ranked by heat, and
this failure mode is silent by construction.

**The ask:** add `stop_reason` to `mine_sessions.py`'s event stream and re-cut. It is one field
on records the script already walks, and it produces the table above plus turns-per-prompt and
"did the agent stop, or was it stopped."

---

## 2. A verified primitive both lanes can use

Neither proposal models a session's *state*, and the transcript supports it directly. Checked
today against the records rather than assumed:

| state | signature | resolved by |
|---|---|---|
| working | records appending; last assistant record `stop_reason: "tool_use"` | the file |
| waiting — turn ended | last main-chain record is an assistant with `stop_reason: "end_turn"`, process alive | the file |
| waiting — permission held | a `tool_use` block with no matching `tool_result`, no child process on CPU | the file + `ps` |
| silent — long call | the same unresolved `tool_use`, but a child process *is* running | `ps` |
| ended | process gone | `ps` |

Two observations that make this safe to build on:

- **`stop_reason` cleanly separates working from stopped.** One 682-record session carried 133
  `tool_use` against 11 `end_turn`.
- **In 36,263 tool-use blocks across the corpus, every single one eventually received a result —
  zero orphans in any finished transcript.** So an unresolved call in a *live* transcript is
  never a lost record; it is always a call that has not come back. The only open question is
  whether something is working on it, which is a `ps` question and nothing else.

Denials are visible verbatim in the result text (`"The user doesn't want to proceed with this
tool use. The tool use was rejected…"`, 26 instances in a 200-file sample), so an approve/deny
outcome is recoverable after the fact — which the field study's "five meshwork calls were denied"
figure could be generalised from across all tools.

The one genuinely ambiguous pair is *permission held* vs *long call*, and it needs the process
walk. Where the walk cannot attribute a child — a remote call, a detached process — the honest
report is "silent, N min" rather than a guessed "waiting," because a false wait trains the true
one out of the reader.

---

## 3. The asks

Ranked by value ÷ cost. Lane tagged.

### A1 — Make `clock` a parameter, not only an environment pin **[analytics]**

`MW-S1` registers `clock` as a one-row table honouring `MESHWORK_TODAY`, and **every** view takes
`now` from it: `facts.age_h`, `idle_h`, the `spans` open intervals, `hazard`'s bins, `flow`'s
calendar, `p_win`. The plumbing already converges on one row.

So one flag — `q --as-of <stamp>`, or a `[stats] as_of` override — makes the entire thirteen-view
layer answer *as of any moment in the store's history*, with no change to a single view body.

What that buys, in rough order of value:

- **Goldens at more than one stamp.** Today the conformance corpus pins one `now`. Bugs in age
  arithmetic, in the hazard's follow-up equalisation, and in `flow`'s day bucketing are exactly
  the class that a single stamp hides.
- *"Was the residual hazard always flat?"* becomes a query rather than a script, which is the
  falsifier `PROPOSAL-prioritization.md` §8 already commits to testing after 60 days of store age.
- *"When did this repo last sit under 8 unanswered asks?"* — the trajectory version of every
  number in `pulse`, which is what a refresh actually wants.
- It removes the single largest reimplementation burden from any external reader. Without it,
  anything that wants history has to re-derive every aged quantity itself, which is precisely the
  second implementation the analytics proposal's §9 was written to avoid.

**Cost:** a flag, a substitution at registration, and a golden at a second stamp. This is the
highest leverage-to-cost item I found in either proposal.

### A2 — The conformance corpus needs a multi-store fixture **[analytics]**

Rung 0 blesses `expected/views/*.json` **from the golden store** — one store. But §6.2 of the
same proposal states that three families are wrong-by-construction in a single-repo load:
`asks_in_open` is 0 in every store, `needed_by_foreign` is 0 everywhere because the foreign
dependent is not loaded, and a cross-repo mention resolves to nothing.

Therefore: **a second implementation that got all three completely wrong would still pass
conformance**, because every expected value is zero. "Views as contract" is untested for exactly
the third of the layer that carries the portfolio's real structure — 161 cross-repo edges, 82 of
them `answers`, 32 open obligations.

**Ask:** add two small linked stores to `fixtures/conformance/` — an ask from A to B, an answer
in B, one cross-repo `needs`, one handoff naming a foreign id — and bless the union views over
them. **Cost:** a fixture and a second bless. It is small now and expensive after a reader has
shipped against the single-store expectations.

*Provenance note: this is an argument from the proposal's own §6.2, not an observed failure — I
have not written a second implementation to demonstrate it.*

### A3 — Measure the owner's latency; re-cut the study's ranking **[field study]**

§1 above, in full. **Cost:** one field added to an existing script.

### A4 — Price the findings with `message.usage` **[field study]**

The study read 449 transcripts and extracted verbs, errors, prompts, hand-edits and `--help`
probes — but not **cost**, which sits in every assistant record as input / output / cache-read /
cache-write tokens. Meanwhile the owner's stated constraint is explicitly tokens (ruling
2026-08-10, *"i only have so many tokens"*), `PROPOSAL-prioritization.md` gates its rungs 5–6 on
a token measurement it plans to take out of band, and the setup-cost matrix already proved the
method.

The study's findings are ranked "by what they cost the owner" — qualitatively. With `usage`
attached, each becomes a number:

- what the 18-minute compile against a 5-minute `RUN_TIMEOUT` actually cost, across 43 of 58
  leras verifies;
- what the six-session repetition cost;
- what 723 hand-edits cost in turns and tokens;
- what a session that never loaded the skill costs versus one that did — the study's sharpest
  behavioural finding (61% never load it) with a price on it;
- cache hit rate per session, which is the difference between a cheap long session and an
  expensive one and is currently invisible everywhere.

I would expect this to re-order the Tier 1 / Tier 2 split. **Cost:** one pass over transcripts the
script already parses. It also retires the "token half of rung 0" that
`PROPOSAL-prioritization.md` §10 asks for authorisation on — same data, same pass.

### A5 — Pin the closure convention in FORMAT.md **[analytics, spec]**

`flow` and `hazard` count `done` by *current* status, so a reopened task is not a closure. The
09-01 prioritization baseline used `min(→done)`. Appendix B records the difference as **21 tasks
in sazed alone**.

Two documents in the same repo already disagree by 21 on one store, the disagreement lives in an
appendix caveat, and a third implementation is being designed right now. This is the
`mw-r6g9bhe` lesson exactly — two surfaces each inventing their own rule for the same quantity —
and it is one sentence in the spec to settle permanently. Same treatment for date-only stamps
rounding to midnight, which is currently also only in Appendix B.

### A6 — State the `sessions` view's sample bias in the view's own documentation **[analytics]**

`sessions` derives identities from comment authors and `claimed by` log notes. But the field
study measures `start` at 302 calls against `close` at 597 — **half of all closed work was never
claimed** — and the UI baseline found 8 of 11 live sessions holding no claim at all.

So `sessions` measures a heavily biased minority of actual sessions, and
`attention.touched_since_move` inherits the same bias. Both are still worth having; neither
should be read as coverage, and a third-party reader joining to them needs to be told. I would
also not over-invest in `attention` as the F7 proxy: it found 13 tasks, and the transcript side
of the same question is far richer.

### A7 — Agree one event envelope across the store and the transcripts **[both lanes]**

`mine_sessions.py --events` emits a session event stream. The `events` view emits a store event
stream. They are two shapes for the same idea, and anything wanting both on one timeline — *what
was this session doing in the ten minutes before it closed that task?* — has to reconcile them by
hand.

Agreeing `{repo, gid?, kind, at, actor, note}` now costs close to nothing and makes three things
nearly free: the reader's live stream, a replay of any session against the store state it saw,
and the field study's own ability to join its findings to the store rather than citing them by
session-id prefix.

### A8 — Break out subagent cost from main-chain cost **[field study]**

The study counts tool calls "on every chain, subagents included," which merges a session that did
the work itself with one that fanned out to six readers. Fan-out is a large token multiplier and
one of the few levers the owner directly controls; the study's own method used exactly this
pattern (39 sessions read by analyst subagents) and cannot price it. `isSidechain` is on every
record and separates them in one line.

---

## 4. What has a downstream consumer waiting on it

Useful for ordering, because these read as hygiene and are actually dependencies. From
`portfolio/PROPOSAL-meshwork-ui.md`:

| item | who asked | what it blocks |
|---|---|---|
| **MW-S14** — reads never prune (narrows mw-chcqk6g) | analytics §12.4 | the reader's whole feed. Until ruled, an external reader must make nine per-repo `q` calls instead of one `portfolio q`, purely to avoid mutating `sequence.md` on load. A reader that owns no state cannot prune the owner's overlay |
| **Tier 1 #2** — `--to`, `--answers`, `--relates`; `set --body/--parent/--from` | field study | the reader's *entire* write surface, not just ergonomics. `FORMAT.md` requires third-party writes to go through the binary, so a missing flag is not a hand-edit workaround — it is a field no external tool can ever set. This argument is stronger than the 723-hand-edits one and the study does not make it |
| **MW-S10** — `set --handoff` mints a log line | analytics §12.6 | handoff age. Without it `moved_since_activity` must use the task's last activity as an upper bound, so every consumer of the D3 staleness check inherits an approximation |
| **A1** — parameterised `clock` | this document | any historical view; the reader's time axis |
| **A2** — multi-store fixture | this document | the only test that a second implementation got the cross-repo columns right |

---

## 5. Where I would push back on the current ordering

**The "what to do first" list has no measurement on it.** Field study §4 names four fixes — the
inbox, the write flags, the DSL/timeout, load the skill with prime. All four are right. But A3
and A4 are each about a day, they use scripts that already exist, and they would plausibly
re-order those four by putting a number on each. With both lanes kicking off now, I would put the
two measurements **in front** rather than beside.

**`stats` (analytics rung 4) is ranked above the fixture (A2), and I think that is backwards.**
Rung 1 — registration — is the rung that gives every session the whole metric layer the day it
lands, and it is correctly early. But `stats` is a presentation of numbers already reachable by
`q`, whereas A2 is the test that the numbers are right in the one region nothing else tests. If
one of them slips, slip `stats`.

**One thing I would not change.** `MW-R12`/`MW-R13` — age never boosts, past-triage work is a
decision queue — is the most disciplined call in either proposal, and the measurements above do
not disturb it. Agent *wait* time is a latency defect in the session layer; it is not an argument
for aging tasks in the store, and it should not be allowed to become one.

---

## 6. Provenance

Following the house evidence rule, separated by how much each claim is worth.

**Observed this session, on this machine, from the records:**

- Everything in §1's table, and the §7 method that produced it.
- §2's state machine: `stop_reason` values and their counts; **36,263 tool-use blocks with zero
  unresolved results** across the sampled corpus; the verbatim denial text.
- The corpus span, 2026-07-26 → 09-07, 569 transcripts.

**Read from the three documents, not independently verified:** the 21-task closure divergence
(Appendix B), the `start`/`close` ratio and every other field-study figure quoted, the view costs,
the §6.2 zero-by-construction claims, the scoring weights.

**Argued, not observed:** A2's conclusion that a wrong second implementation would pass
conformance — that follows from §6.2 plus a single-store bless, but I have not written a second
implementation and run it. A4's expectation that pricing re-orders the tier split is a prediction.

**Not checked at all:** I did not run `verify_meshwork.sh`, any test, `mine_views.py`,
`mine_rank.py` or `mine_sessions.py`; I did not exercise the binary; I did not validate the
DataFusion version analysis in §7 of the analytics proposal. No file in this repo other than this
one was created or modified, and no store was read through a `portfolio` verb.

**Drift:** the corpus was live during the scan — sessions were appending while it ran, and an
earlier pass differed by units (1,228 vs 1,233 waits). The table above is a single consolidated
pass; a re-run will drift the same way, exactly as Appendix B notes for the store.

---

## 7. Reproduction

```
for each ~/.claude/projects/*/*.jsonl:
    seq = records where not isSidechain
    for each assistant record r in seq with r.message.stop_reason == "end_turn":
        a = r.timestamp
        b = timestamp of the next record in seq that is
              type == "user" and not isMeta and userType != "tool_result"
              and has non-empty text content
            — but stop the scan at the next assistant record
              (the agent continued on its own: not a wait)
        if b exists and b > a: emit wait(file, a, b)

live-elsewhere-at-stop(w) := any record in another transcript within [a, min(a+5m, b)]
owner-active-minutes(w)   := |{ distinct minutes in [a,b] carrying a record in another file }|
```

Two judgment calls worth reviewing before this is adopted. **Stopping at the next assistant
record** means a session the agent resumed on its own is not counted as a wait; that is
conservative and pushes the total down. **Excluding `isMeta` user records** drops
hook-injected and system content, which is right for "the human typed something" but should be
re-checked against the queued-prompt handling the field study already built — a prompt typed
while the agent was mid-turn is a *zero* wait, and if those records are shaped differently on
this machine than the study assumes, the median moves and the tail does not.

---

## 8. One closing observation

The three documents in this repo are unusually good at measuring the tool and the agents. All
three treat the owner as the fixed point the system is measured against — the one who erupts,
steers, rules, and gets interrupted. The number in §1 is the first evidence I have seen that the
owner is *also* a resource in the system with a queue in front of it, and that on 34 of 44 days
that queue was the longest one on the machine.

That reframing is worth more than any single ask above, and it belongs to the field-study lane:
`prime` is a digest handed to an agent at the start of a session, and the most expensive state in
the corpus is created *after* it has been read, by an agent that has stopped, in a repo nobody is
looking at.
