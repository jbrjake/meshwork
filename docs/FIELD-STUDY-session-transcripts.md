# Field study — how agents actually use meshwork

**Status: findings + recommendations, unruled.** Source: every Claude Code session transcript on
this machine for the nine registered repos plus the code root, 2026-08-04 → 2026-09-06. The
transcripts are the closest thing meshwork has to user testing and telemetry: 449 sessions, 256 of
which called meshwork, 4,270 meshwork invocations, 2,067 human prompts. Method in §5; every claim
about the binary was re-checked against v0.4.0 and the reproduction log is §6. Companion scripts:
`scripts/mine_sessions.py` (score + event stream), `scripts/mine_telemetry.py` (corpus tables),
`scripts/render_session.py` (read one session).

The one-paragraph version: **the tool's local loop works and is trusted** (show → start → work →
close on a real verify, near-zero waives, honest disclosures), **and the cross-repo half is
structurally invisible** — the inbox is capped, suppressed and mis-queried, the two fields that make
asks work have no CLI path, and the previous agent's `handoff:` outranks the owner. Almost every
owner eruption in the corpus traces to one of those three things or to the agent inventing owner
authority. The rest is discoverability tax: 408 `--help` probes, 723 hand-edits of task files, 891
shell reads of the store.

---

## 1. Telemetry

### 1.1 Corpus

| repo | sessions | used meshwork | calls | errors | owner said "meshwork"/an id after turn 1 | emphatic prompts | hand-edits |
|---|---:|---:|---:|---:|---:|---:|---:|
| sazed | 164 | 69 | 1,245 | 52 | 24 | 288 | 256 |
| meshwork | 47 | 42 | 859 | 60 | 24 | 53 | 71 |
| marasi-applied-r-and-d | 27 | 24 | 562 | 32 | 12 | 93 | 111 |
| leras | 55 | 31 | 487 | 43 | 11 | 82 | 91 |
| tensoon | 39 | 33 | 400 | 25 | 3 | 24 | 50 |
| marasi | 25 | 21 | 318 | 23 | 0 | 9 | 16 |
| wyndam | 10 | 7 | 181 | 5 | 0 | 3 | 29 |
| code root | 45 | 14 | 102 | 14 | 20 | 34 | 69 |
| oreseur | 7 | 5 | 58 | 1 | 5 | 4 | 8 |
| portfolio | 30 | 10 | 58 | 1 | 11 | 49 | 21 |

"Emphatic" = a run of two or more ALL-CAPS words, `!!`/`??`/`?!`, or profanity / explicit
exasperation. It is the owner's emphasis register, not only anger: 599 of 2,067 prompts (29%), in
172 sessions; 111 of the 256 meshwork-using sessions contain at least one. 108 emphatic prompts
were followed by a meshwork mutation before the next prompt.

Weekly trend (sessions / used meshwork / calls / errors / `--help` / hand-edits): W32 71/14/181/6/5/72
→ W33 91/44/634/52/78/169 → W34 75/42/642/50/36/122 → W35 86/58/829/51/105/131 → W36
124/98/1,984/97/182/229. Usage tripled in the last week; the error rate held near 5% of calls;
help probes and hand-edits scaled linearly with usage — none of the friction below is being
learned away.

### 1.2 Verbs

| verb | calls | | verb | calls |
|---|---:|---|---|---:|
| show | 914 | | ready | 178 |
| close | 597 | | search | 170 |
| set | 586 | | portfolio q | 70 |
| comment | 532 | | verify | 68 |
| add | 522 | | dep | 48 |
| lint | 488 | | why | 40 |
| q | 431 | | tree | 39 |
| start | 302 | | portfolio next | 31 |
| prime (explicit re-run) | 185 | | reopen / block / init | 24 / 23 / 20 |

Never or almost never: `blocked` 7, `import` 7, `attach` 3, `drop` 8, `portfolio ready` 8, `mirror` 0.
`start` runs at half the rate of `close` — half of all closed work was never claimed.

**Verbs agents typed that do not exist** (22 calls): `addressed` 6, `portfolio show` 6, `list` 5,
`portfolio inbox` 4, `inbox` 2, `next` 2. Every one of them is a request for the same thing: *the
inbox, in full*. Clap's suggestions for them are `add`, `set`, `lint`, `dep` — all wrong, two of them
mutating.

**`--help` probes: 408, 9.6% of all meshwork calls.** `add --help` 121, bare `--help` 84, `set` 64,
`comment` 28, `dep` 23, `portfolio` 22, `q` 14, `close` 10. Sessions in repos that use meshwork
daily still relearn `add`'s flags live. The verbs most probed are the ones with the most flags
and the ones (`portfolio`, `q`) whose sub-surface is not visible from the top-level help. No
`--help` text of any verb mentions `to:`, `answers:`, `addressed`, or `inbound` — the ask vocabulary
exists only in SKILL.md, and **the skill was loaded in 99 of the 256 sessions that used meshwork**
(39%; 80 of the 177 sessions with ten or more calls). In those 177 heavier sessions `start` appears
in 92 and `search` in 42.

### 1.3 Errors

256 errors in 4,270 calls (6.0%). By kind: close refused 74, unclassified non-zero exit 74, shim
not found 23, unknown verb 22, bad arguments 22, id not found 13, YAML parse 11, panic 4.

Top messages, normalized:

| n | message |
|---:|---|
| 28 | `refusing unapproved verify for ID (MW-E5, DESIGN §12b)` |
| 9 | `ID not found in PATH` (an id prime had just printed with a sibling prefix) |
| 7+5+4+4 | `no such file or directory: ./meshwork` / `docs/meshwork/meshwork` / `command not found: meshwork` |
| 6+2+2 | `refusing malformed verify … bad arg token: --test` / `-p` / `--release` |
| 5+5+5+2+2 | `unrecognized subcommand 'inbox'` / `'help'` / `'show'` (under portfolio) / `'next'` / `'addressed'` |
| 5+2 | `mapping values are not allowed in this context` (hand-authored or batch YAML) |
| 3+2+2 | `unexpected argument '--body'` / `tip: to pass '--body' as a value` / `tip: to pass '--to' as a value` |

### 1.4 Close refusals and how they were resolved (71)

| refusal | n | then |
|---|---:|---|
| unapproved verify (MW-E5 gate) | 30 | 25 closed with no store edit (i.e. `--approve`), 5 never closed |
| verify actually failed | 25 | **10 closed after `set --verify` rewrote the verify**, 5 closed after fixing the work, 9 never closed, 1 after a hand-edit |
| malformed verify (DSL parse) | 15 | 10 repaired via `set --verify`, 4 never closed, 1 hand-edit |
| no verify at all | 1 | never closed |

`--waive`: 7 uses in the whole corpus. `set --verify`: 203 uses. Ten of twenty-five genuine
failures were resolved by changing the verify rather than the work; in the sessions read closely,
every such repoint was disclosed in a comment and the stated reason (the old verify was green before
the work) was correct — which relocates the defect to `start`, whose red-check should have caught
the vacuous verify first (§2.6).

### 1.5 Prime, asks, and the store outside the CLI

- Prime was injected in 298 sessions. **100 of them never made a meshwork call.** Of the 198 that
  did, 94 never `start`ed anything, 39 started on prime's `next →`, 65 started elsewhere. Prime's
  byte budget holds: p50 3.9 KB, p90 5.0 KB, max 5.5 KB, none over 6,144.
- 86 sessions had asks addressed to their repo in prime; 46 (53%) touched a foreign id or hopped
  into a sibling store at any point.
- **Hand-edits of task files: 723 in 191 sessions** (Edit 517, shell/heredoc 130, Write 76; 54 on
  archived files). Against 1,640 CLI field writes (`set` + `comment` + `add`), ~30% of all writes
  to the store bypass the CLI. The close reads (§2.3) trace nearly all of them to fields with no
  flag.
- **Store read with shell instead of the CLI: 891 calls in 247 sessions** — grep 293, ls 190, head
  146, cat 92, sed 68, find 11. `search` was used in 100 commands. SKILL.md says never `grep -r`
  the store; the corpus greps it three times for every `search`.
- Cross-repo hops (`cd ../x && ./docs/meshwork/meshwork …`): 108 calls in 31 sessions; 52 into
  sazed. Verbs after the hop: show 54, search 41, q 21, prime 12, comment 7.
- meshwork's share of a session's tool calls: median 10%, p90 22%. Comments: 499, 170 of them via
  `@file` or stdin (long prose). Five meshwork calls were denied by the owner; two were a `set
  --handoff` or `--body` carrying a 🔴 banner.

### 1.6 What the owner costs the agents — waits, and a session-state primitive

Everything above measures what agents cost the owner. The inverse is in the same records and
neither the scoring in §5 nor any finding in §2 can see it: a session that finished its turn
and sat produces no prompt, no call, no error and no edit, so it scores zero. This cut adds
`stop_reason` to the event stream (`mine_sessions.py`, whose header states the algorithm) and
measures the wait after every `end_turn` — from the stop to the moment the next prompt was
*typed*, so a prompt queued mid-turn is a zero wait, and a turn the agent continued on its own
(a hook, a task notification) is not a wait at all. Same corpus as §1.1, re-cut 2026-09-08; every
transcript on the machine (590) feeds the "elsewhere" index.

| measure | value | denominator / note |
|---|---|---|
| corpus | **484 transcripts, 2026-07-31 → 09-08 (40 d)** | the nine registered repos + the code root |
| main-chain messages by stop reason | **end_turn 1,476 · tool_use 60,382** | one message counted once, however many records it spans (up to seven) |
| `end_turn` answered by a prompt | **731**, in 258 sessions | 260 of them typed mid-turn and delivered at the stop: zero waits |
| `end_turn` the agent went past on its own | 314 | woken by a hook or a task notification; not a wait |
| `end_turn` with no following prompt | **431** | the transcript ends there — 431 of 484 sessions; a deliberate close and a walk-away look the same in the record |
| median wait | **1.6 min** (2.4 over the 471 typed after the stop) | the median says nothing about the problem |
| p75 / p90 / p99 / max | 4.9 min / **19.2 min** / 14.6 h / 166.7 h | |
| **total agent idle waiting on the owner** | **545 h** | 91% of it in waits > 1 h; **four waits over 24 h — sessions resumed days later — hold 287 h of it** |
| waits ≥ 15 min | **85**, 521 h idle | **65 (76%) began while another session on this machine was live**; on **29 of 40 days** |
| waits ≥ 1 h | **41**, 499 h idle | 31 (76%) began while another session was live; on 23 of 40 days |
| minutes *another transcript was live* inside those waits | 152 h (≥ 15 min) · 142 h (≥ 1 h) | any record elsewhere — which includes other agents working unattended |
| **owner-active minutes** inside those waits | **81 h** (≥ 15 min) · **76 h** (≥ 1 h) | the owner attending another transcript: within five minutes after a prompt there, or between two prompts there under fifteen minutes apart |
| median owner-active minutes per long wait | 7 min (≥ 15 min) · **18 min** (≥ 1 h) | |
| long waits holding ≥ 30 min of owner activity elsewhere | 17 of 85 (20%) · **16 of 41 (39%)** | |
| what a prompt after the first followed | **end_turn 649 · tool_use 1,015** · none 49 | the agent had stopped, or was stopped: 260 of the 1,015 were queued for the next stop; the rest answered a permission hold or interrupted a call |
| idle hours by repo | marasi 182 · code root 92 · portfolio 92 · leras 82 · sazed 75 · meshwork 13 · lab 8 · tensoon 2 | the marasi figure is one 167-hour resumption |

**Three readings.** First, the number that is recoverable is not the 545 h. Idle is latency, not
spend — a stopped session burns nothing — and more than half of it is four sessions the owner
came back to days later. The recoverable part is the **76 h** inside hour-plus waits during
which the owner was demonstrably working in another session, and the sharpest line is the last
column of the ≥ 1 h row: **sixteen of forty-one hour-plus waits held half an hour or more of the
owner's attention elsewhere.** Second, "live elsewhere" and "owner active" are different numbers
and the ask that requested this measurement (`ASKS-analytics-and-field-study.md` §1) conflated
them: its 153 h reproduces here as 152 h, but it counts any record in another transcript, and
half of those minutes are other agents working with nobody at the keyboard. Its 1,233 waits and
1,594 unanswered turn-ends are consistent with counting records rather than messages. Third,
**the owner is stopped-by more often than stopped-for**: 1,015 prompts arrived while the agent
was mid-call against 649 after it had stopped, and a quarter of those were queued for the next
stop. The owner already uses the queue; what the queue cannot reach is a session that stopped in
a repo nobody is looking at.

None of this is an argument for aging in the store. It is a latency defect in the session layer
(the ask's §5 says the same), and it lands in this study because prime is read once, at the
start, and the most expensive state in the corpus is created after that read — by an agent that
has finished, in a repo the owner has moved on from.

**A session-state primitive, checked against the records.** The transcript supports five states,
and the two observations that make them safe to build on both hold on this corpus:

| state | signature | resolved by |
|---|---|---|
| working | records appending; last main-chain message `stop_reason: tool_use` | the file |
| waiting — turn ended | last main-chain message `stop_reason: end_turn`, process alive | the file |
| waiting — permission held | a `tool_use` block with no matching `tool_result`, no child process on CPU | the file + `ps` |
| silent — long call | the same unresolved `tool_use`, but a child process *is* running | `ps` |
| ended | process gone | `ps` |

`stop_reason` cleanly separates working from stopped (1,476 `end_turn` against 60,382 `tool_use`
messages). And **in 68,822 `tool_use` blocks across every transcript not touched today, every
one received a `tool_result` — zero orphans** — so an unresolved call in a live transcript is
never a lost record; it is a call that has not come back, and whether anything is working on it
is a `ps` question and nothing else. Denials are verbatim in the result text: 181 declined tool
calls across all tools, the five meshwork ones of §1.5 among them. The one ambiguous pair is
*permission held* against *long call*; where the process walk cannot attribute a child, the
honest report is "silent, N min", not a guessed "waiting".

---

## 2. Findings

Ranked by what they cost the owner. Each carries the sessions it rests on (repo/id-prefix) and its
status at v0.4.0. Analyst reports with turn-level evidence are in the study's scratch directory;
the quotes below are verbatim from transcripts.

### 2.1 The inbox is invisible three ways (plus two smaller traps), and the eruptions follow from it

Every "you ignored the other projects" eruption in the corpus — sazed/72773264, sazed/1b167145,
sazed/94b76cfa, marasi-applied-r-and-d/58b837fb, tensoon/f1040972 — has the same mechanism
underneath it, and it is the tool's, not the agent's:

1. **Row cap.** `prime` prints `addressed to this repo (N):` then three rows and `… and N more
   addressed` (`ADDRESSED_ROWS = 3`, oldest first). `ready` caps at five. On 2026-09-01
   `wy-epwnd25` had sat below the fold for 13 days (leras/5b3b9348). No session in the corpus ever
   followed the "and N more" line with a command that expands it — there is none to name.
   **This repo's own prime, today, shows 3 of 5 asks addressed to meshwork; the DSL-gap ask
   (`leras#le-yppwtpq`) and the close-corruption ask (`marasi-applied-r-and-d#ar-gfd8g38`) are the
   two below the fold.**
2. **Suppression by intent.** `src/addressed.rs` drops an ask the moment *any non-dropped* task
   anywhere carries `answers: <gid>` — an open placeholder counts. On 2026-09-04 sazed's prime said
   `(3)` while `portfolio q` returned 10 open asks; `ar-hcvyxy5` — the one the owner called
   "LITERALLY THE TOP OF THE QUEUE … THIS IS LIKE THE SIXTH FUCKING TIME" — was hidden by
   `sa-ewaw5vy`, an open ANSWER task filed 72 minutes earlier (sazed/1b167145). Four of six
   suppressed asks in sazed/94b76cfa were hidden by sazed's own open tasks. Filing an intent to
   answer erases the ask from the addressee's view before a line of work exists, and inbox-zero
   can be reached by frontmatter alone (sazed/9ee85e0f added `answers:` to an archived task and
   watched the section empty).
3. **Wrong-store query.** `addressed_to` on the local `tasks` table is the *outbound* `to:`. A local
   `q … WHERE addressed_to = '<me>'` is well-formed and returns the repo's own asks (sazed/72773264
   opened on this and mistook it for the inbox) or `(0 rows)` in a repo whose inbox is 10 deep
   (sazed/1b167145). Only `portfolio q` sees sibling stores. No hint is printed. And `answers` is
   an edge kind, not a column: `Schema error: No field named answers` (sazed/94b76cfa,
   sazed/7e29128c) — the field that retires an ask cannot be queried, so the agent fell back to
   `grep -l answers docs/meshwork/*.md`.
4. **Two smaller traps.** `ready` orders by `coalesce(seq, 999999)`, so an answering task filed
   without `seq` sorts to the bottom and never reaches prime (`sa-399rbej`, sazed/7e29128c); `add`
   does not require `seq` and lint does not warn. And an ask's `verify:` names paths inside the
   *asker's* repo (`contains experiments/dock-firmware/PROFILE.md /…/`), so the repo expected to
   answer it cannot run it — "THEIR VERIFY STRING: absent" (sazed/7e29128c).

Prime also *places* asks below `weather` and above `next →` with a source comment that they "never
displace next" (`src/cli/prime.rs`), and `ready` calls them "a footnote, never a second worklist"
(`src/cli/query.rs`). The owner's stated model is the opposite: "YOUR MESHWORK PRIME FUCKING TOLD
YOU LERAS HAD MULTIPLE TASKS READY FOR YOU! WHAT THE FUCK DID YOU FOCUS ON INSTEAD" (sazed/72773264).
That is a product-intent conflict, not an agent defect. Nothing tells the asker either: an ask
answered in sazed's store on 08-30 was still `open` in leras on 09-04, and an agent stated the
false belief that `answers:` "will surface in the other repos' prime" (sazed/1b167145).

Outbound asks are second-class in the other direction: a `to:` task sits in the *sender's* own
`ready` list like any work item, so the compensating idiom is prose — three handoffs in one lab
session open "Nothing to build here — this is an ask addressed to <repo> and it closes when
<repo> acts" (marasi-applied-r-and-d/55d99d82). That is a status being spelled by hand. In the
four lab sessions read, `answers:` was never used; asks were closed against `contains`/`exists`
greps the author had just satisfied.

Status at v0.4.0: all mechanisms confirmed in source. `mw-r6g9bhe` (age unanswered asks in the
headline) is open and addresses part of this.

### 2.2 The previous agent's `handoff:` outranks the human

The steering artifact is the `»` handoff of whichever task the agent opens first — usually prime's
`next →`, but the effect is the same when the agent reaches the task by `show`. In every group A
session the agent's first action was that task's handoff, written by a previous session — while
the owner's typed instruction sat unconverted. The handoffs are
escalating: `🔴🔴 START HERE. THIS IS THE ONE THING. Do not touch anything else first` (60 lines,
sazed/94b76cfa); `START HERE. This is seq 1 and it outranks every ask in the store, including the
two addressed to siblings` (marasi-applied-r-and-d/97b2e947) — a handoff instructing the next agent
to demote inbound asks. The agent's own diagnosis after the eruption is the finding:

> "That handoff was written before leras shipped `abd8a37`. It was a previous session's voice
> describing a world that no longer existed, and the four asks it didn't mention were sitting in
> `ready` directly underneath it." (sazed/72773264)

The strongest instance: the owner's opening prompt was "YOU NEED TO DEAL WITH THE MANY REQUESTS
FROM OTHER PROJECTS THAT YOU HAVE BEEN FAILING TO ADDRESS"; the agent's first call was `show` on
prime's lead, whose handoff read `» 🔴🔴 DO THESE TWO THINGS BEFORE ANY OTHER WORK. Both … were
missed on 2026-09-04, which cost that whole session`, and it answered "I'll start with the pin"
(sazed/7e29128c). A banner that scolds a previous session by date reads to the next agent as an
owner ruling; nothing in `show`'s output distinguishes the two.

Nothing marks a handoff's author or age; `weather` prints `[stale: 27d]` on `doing` tasks and lint
warns `handoff-stale` on closed ones, but a handoff whose world moved renders in the loudest slot
on the page with no signal. Prime is also read once: it is injected rather than typed, and agents
do not re-run it when the question changes — "THERE WAS A FUCKING MESHWORK TASK FOR THIS … IT WAS
YOUR CURRENT TASK IT WAS IN PRIME!" landed on an agent that had filed a duplicate of its own
current task without searching the store (sazed/525c8a3a). Explicit `prime` re-runs occur in 99 of
256 sessions. Meanwhile the *other* direction fails too: three leras sessions in one
day wrote zero handoffs (leras/C group) — the handoff migrated into commit subjects and `RELAY
ANSWER` comments, because the commit is already being written and the handoff is a second call.
Users have started hand-maintaining meta-tasks whose body is a prose mirror of the inbox with the
SQL to regenerate it (`sa-erwjmcm`, sazed/1b167145): they are building the report prime does not give
them, inside a task.

### 2.3 The fields that make asks work have no CLI path — hence 723 hand-edits

At v0.4.0 `add` and `set` have no `--to`, `--answers`, or `--relates`; `set` has no `--body`,
`--parent`, `--from`; `set --docs` appends and cannot replace a bad anchor. SKILL.md's Rules
section says *"Every field also has a CLI path."* It is not true for the two fields the cross-repo
mechanism depends on. Consequences observed:

- Three sessions independently discovered `add --to` does not exist (`tip: to pass '--to' as a
  value, use '-- --to'` — clap's tip reads as "the flag exists, you quoted it wrong"), probed
  `--help` twice, loaded the skill mid-eruption, and hand-edited frontmatter (97b2e947, 58b837fb,
  45888824). Every `to:`/`answers:`/`relates:` write in the three leras ask sessions — 12 of them —
  went through `Write`, `perl -0pi`, or a python heredoc; one produced a lint error (`relates:`
  scalar), one voided a verify approval and made the next session's `close` refuse.
- `answers:` is written late or never: leras/1e16364c closed five answering tasks and wrote zero
  `answers:` lines; leras/5b3b9348 spent ~10 minutes backfilling eight by grepping archived bodies
  for sibling ids. Until then every sibling's prime still showed the asks as owed.
- Satisfying a `description-size` warning takes six hand-edits because `set` has no `--body`
  (97b2e947); two ANSWER tasks were `Write`-rewritten whole seconds after `add` created them
  (1b167145).
- Re-homing one task from the tool repo to the portfolio store cost ~25 operations — init, re-add,
  copy body, 5×`dep rm` + `dep add`, comment, drop, ten prose edits, two `rm`s of archived files —
  because there is no `move` (code/533950bf).

The hand-edits then break things: `error[parse] … mapping values are not allowed` (11 in the
corpus), and `close` on a hand-edited task refuses (`is invalid … repair before closing`). They are
not laziness: the one workaround an agent tried (`set --body "$(cat <<'BODY' …)"`) was blocked by
the repo's own anti-heredoc hook, and `set` has no `--body` anyway (sazed/7e29128c).

The cost of the vocabulary living only in SKILL.md is measurable. In sazed/51e60dbd the owner
erupted five consecutive times ("NOT AS A RELAY USE FUCKING MESHWORK" → "NO YOU DON'T FILE IT IN
THEIR STORE YOU FILE IT IN OUR STORE WITH A TO: LINE" → "YOU HAVE THE MESHWORK SKILL FUCKING USE
IT" → "WHAT THE FUCK HOW HAD YOU NOT LOADED IT?") while the agent searched `--help`, `portfolio
--help`, `add --help`, `SELECT * FROM repos`, `SELECT * FROM tasks LIMIT 1 --json` (to read the
column list off a sample row), `set --help`, and a sibling's raw frontmatter. When the skill finally
loaded, behaviour flipped inside one minute: `add --batch` with `to:`, DSL verifies, `lint`,
`add --body`. The agent's post-mortem: "I worked from the CLI surface instead, so I never saw the
`to:` mechanism — I went looking for a place to *put* the ask rather than reading how asks are
addressed."

### 2.4 Discoverability tax

- **The sibling lookup is discovered by trial in every session that needs it.** `show le-4k6cz9r`
  on an id prime had just printed as `leras#le-4k6cz9r` answers `not found in
  …/sazed/docs/meshwork` — names the wrong store, offers no route. The recovery ladder was
  `portfolio inbox` (unrecognized) → `portfolio --help` → `portfolio q` → `cd ../leras && ls | grep`
  → `Read` the file by path: five calls and a forbidden grep for what `cd ../leras &&
  ./docs/meshwork/meshwork show le-4k6cz9r` answers in one (sazed/9ee85e0f). Only one session in
  the sazed group used the taught path.
- **Shim path skew and cwd loss.** `./meshwork` vs `docs/meshwork/meshwork` produced 23 shim-missing
  errors; a `cd ../marasi` left `start sa-jdgf9d5` failing on a relative shim, never retried, and
  the task was worked and closed while `open` (sazed/72773264). 12 consecutive failures in one
  loop over sazed at v0.2.0 (code/533950bf).
- **`dep add A B`** was typed positionally in two sessions a week apart; the tool's own success
  line (`ma-j57h3qp add needs ma-48bwr0t`) models the invalid form.
- **`q` has no schema help**; two sessions reverse-engineered the `tasks` column list with
  `SELECT * FROM tasks LIMIT 2` (dumping 1.5 KB body cells) and `pragma_table_info` (not found).
  The schema *error* message does list valid fields — agents recovered in one call when they
  guessed wrong; they could not find the list without guessing wrong first.
- **`show --comments 10`** errors (`--comments` is a boolean); agent fell back to `cat` on the file.
- **`portfolio ready`/`next` mutate** `sequence.md` (autoprune) — three of four portfolio-level
  sessions dirtied a file they had not read, as a side effect of a query.
- **`search` is per-repo and there is no `portfolio search`.** SKILL.md says find prior art with
  `search` and, separately, to `cd` into siblings; an agent did the first and not the second and
  spent a morning re-inventing instruments already specified in a sibling task that carried the
  owner's own comment ("HOW CAN YOU EVEN START TO DO THIS WORK WITHOUT GROUNDING YOURSELF IN THE
  TOOLS?????", marasi-applied-r-and-d/a48dfb6e). **The lab's answer was to build its own
  cross-repo prime**: `scripts/ground.sh` runs `q` against three sibling stores and is now that
  repo's SessionStart hook, having displaced `meshwork prime` entirely.
- **`show` prints a `file:` path that does not resolve for archived tasks** (store root instead
  of `archive/`; reproduced at v0.4.0), so the agent fell back to `find` in a sibling store and
  `Read` on the file — both banned by SKILL.md (marasi-applied-r-and-d/55d99d82).
- **The portfolio repo's store has no shim and no SessionStart hook** (confirmed): the one repo
  that holds cross-repo memory starts every session cold. In a nine-turn, five-eruption hunt for a
  task the owner insisted existed, the agent hand-rolled `q … LIKE` over eight repos and then
  `grep -r`, never typing `search`, which had shipped five days earlier (portfolio/e74707a4). The
  item had never been filed.

### 2.5 Defects (items 1–3 and 10 reproduced at v0.4.0, 4–7 confirmed in source, 8–9 and 11–12 observed in transcripts)

1. **`close` corrupts a task whose `handoff: |` block contains an unindented blank line.** The key
   and the first paragraph are removed; the paragraph after the blank line is left orphaned inside
   the frontmatter → `error[parse] … mapping values are not allowed in this context`, and `lint
   --fix` cannot repair it. `set --handoff` writes blank lines indented, so CLI-authored blocks are
   safe; hand-authored ones (legal per SKILL.md) are not. Filed by the lab as
   `ar-gfd8g38` after it bit 97b2e947; reproduction in §6.
2. **`start`'s red-check on an approved legacy-shell verify runs `sh -c <verify>` with no timeout
   and output discarded** (`src/cli/transition.rs`). An unscoped `cargo test` compiles for minutes
   in total silence; the agent sees a hang, kills it, and hand-edits `status: doing` into the file
   (6 of 7 `start` attempts, leras/1e16364c, filed as `le-3m0gpb1` — in leras's store, without
   `to: meshwork`, so this repo's prime will never see it). The DSL `run` path has `RUN_TIMEOUT` of
   five minutes; the shell path has none.
3. **The verify DSL's `run` rejects flag tokens** (`bad arg token: -p` / `--test` / `--release`), so
   `cargo test -p <crate>` is inexpressible and verifies stay on unscoped shell text — measured at
   18 minutes of compile against a 5-minute `RUN_TIMEOUT` (leras/cf057228, filed as
   `le-yppwtpq`); 43 of 58 live leras verifies stayed shell for this reason.
4. **`lint` warns `description-size` on archived, immutable tasks** (18 of 23 in one store); the
   warning is ambient noise scrolled past all session, so the five live ones went unnoticed for
   weeks (97b2e947; loop in `src/lint.rs` has no archive exclusion).
5. **`add` validates no edge target**: `add --from mw-mjwfxn` (typo) minted two dangling edges
   silently; caught at the next `lint` (meshwork/c11bd67e).
6. **Cross-repo `needs:` against an unregistered repo lints clean and never gates** — "the
   cross-repo graph was fiction" (code/60dd6557; `src/lint.rs:227` leaves cross-repo edges to the
   registry). Malformed `sequence.md` bullets (trailing HTML comments) are silently treated as
   prose; discovered by reading `src/registry.rs`.
7. **The canonical shim keys on `CLAUDE_CODE_BRIDGE_SESSION_ID`**; this CLI session exports
   `CLAUDE_CODE_SESSION_ID` and not the bridge variable, so the shim falls back to
   `default_author` and agent work is stamped as the human (wyndam/22f16aee found every sibling
   affected; environment-dependent — bridge sessions do get an author). Open ask `or-9c7n6hp`.
8. **`import`** truncates titles across the frontmatter wrap, splits multi-line verifies into
   fragments (`jit`, `named on arrival`), and absorbs everything between checkboxes into the
   preceding task's body (a 340-line section became one 17 KB body). Every adoption session needed
   a manual repair pass (leras/6f063ba1, tensoon/a3a4b87f, oreseur/2a751b44).
9. `add --batch` frontmatter is strict where `add`'s flags are lenient (`relates:` list-only,
   unknown key `doc`, trailing `---` → `unclosed frontmatter fence`), each mismatch a round trip.
   `q --json` nests under `data.rows`, not `rows` (a scripted sweep broke on it).
10. **Verify authoring has no early failure.** `add --verify` and `set --verify` accept a
    malformed DSL string (`run ./scripts/regression.sh`, `run cargo test --test asks …`);
    `start` warns and transitions anyway; only `close`/`lint` refuse (reproduced: `add … --verify
    'run cargo test -p leras topn'` minted a task; `verify` then refused it). `add --help` still
    describes `--verify` as a command `close` runs via `sh -c`, which is what invites the shell
    shapes the DSL rejects.
11. **Every hand-edit of a `verify:` mints `verify-changed-since-approval`**, which then rides
    along in every lint and pre-commit hook output for the rest of the session (2–3 of them for
    hours in two lab sessions) — self-inflicted by the missing flags that forced the hand-edit.
    `doc-missing` fires on an artifact task whose `docs:` target is precisely the deliverable the
    task exists to create (`ar-nw16txf`).
12. **`add --body "<inline>"` is a shell footgun.** Backticks in prose were command-substituted
    by zsh (`command not found: udp`) and the store took the mutilated body without complaint;
    caught by a later grep (marasi-applied-r-and-d/86796a4d).

### 2.6 The verify economy

The MW-E5 trust gate inverts the safety order in practice. The cheap pre-work check (`start`'s
red-check) is *skipped* for any verify another session or clone wrote ("note: red-check skipped —
verify unapproved for this clone"), and the expensive post-work one is waved through with a flag:
in four dogfooding sessions, **43 of 44 closes carried `--approve`** after two refusals were ever
read; 9 of 9 `start`s skipped their red-check in one session; the one time the red-check fired it
caught a real vacuity (meshwork/1e08d3d4). Where the red-check did run, it paid for itself: one
audit found five vacuously-green verifies and 28 that would exit 127 forever because `rg` is a
shell function (meshwork/9b03f0d9); another refused to close an umbrella whose zero-open-children
verify was green because three children had never been filed (leras/cf057228).

Verifies that grep prose rot silently and fail open: `sa-87jpgw8` sat open for 22 days because a doc
rotation moved its grep target into `docs/archive/` ("wtf WHY WEREN'T THEY SENT???????",
sazed/51e60dbd); `sa-kanb0gq`'s verify named a file that never existed — the agent's own later
words: "that was me dressing up 'I filed it' as 'I handled it'" (sazed/7e29128c). Lint's warning
channel is dead in the busiest store: 274 `verify-shell` warnings on every run, waved off as
"pre-existing store-wide", so the real signals scroll past with them. A `--waive` reason is
permanent, including one that shipped with a literal `sa-<follow-up>` placeholder, and `--waive`
gets used where `drop` was the verb, landing moot tasks in `done` (sazed/525c8a3a, 51e60dbd).

Self-satisfying verifies exist in the wild and lint does not see them: a doc task closed on `grep -q
MW-E5 REQUIREMENTS.md` after the agent wrote the string it invented (meshwork/c11bd67e); a ruling
task closed on a marker the agent itself had written as a comment four minutes earlier
(leras/1e16364c). Two tasks were closed and committed on a **red gate** through the `tail
<gate.log> && close && git commit` idiom — tail's exit 0 carried the chain — both self-detected,
neither reopened, both logged `→done (verify exit 0)` (meshwork/66cb8e9e, 6a71affa), with a memory
file about the trap present and edited twice in between.

### 2.7 Behavioral psychology

What the agent does with a tracker, as distinct from what the tracker does to the agent:

- **The store is treated as the output of analysis, not its input.** In three of four
  portfolio-level sessions the agent read docs, formed a plan, then minted tasks to record it. The
  owner's spoken instruction never becomes a task: "You said 'sazed should have resolved your
  latest blocker.' That was the session's job. I never wrote it into the store. So every piece of
  work I did all session was untracked" (58b837fb). Advisory sessions file nothing: "I generated a
  decaying, actionable, cheap, red-flagged item and put it in the single place the portfolio says
  nothing lives … I behaved like a commentator instead of an operator" (portfolio/e74707a4).
  Asked to draft a task, the agent pasted the rendered draft into chat; the owner: "just put the
  actual fucking task on disk in meshwork so a new session can execute on it."
- **The store enters the conversation only on the literal word "task".** Across the four
  second-wave lab and meshwork sessions, owner instructions became tasks exactly when the owner
  said "add a task" / "file a ticket" / "set up a task" — and one of those had to be repeated
  verbatim, the filing landing 24 minutes and two reminders later. Rulings, corrections and
  standing rules ("one singular commit", "never edit the README", "never trust a web summary")
  went to memory files, CLAUDE.md, a SessionStart hook and three new shell scripts: six memory
  writes, zero tasks. "Stop" is honoured literally, leaving known-wrong content in two task
  files until a second, angrier prompt ("WHY THE FUCK WOULDNT YOU FIX IT WHEN I TOLD YOU YOU WERE
  WRONG??????", marasi-applied-r-and-d/55d99d82). Corrections that do reach the store are written
  *into* bodies as `**NARROWED 2026-08-29, same day it was filed.**` / `**REFRAMED …**` /
  `RETRACTED —` preambles, or by retitling; there is no amend or retract verb, and the retraction
  prose then competes with the live claim.
- **Filing as avoidance.** "File it immediately, never carry it in your head" gets applied to
  decisions that needed the owner *now*: four architecture forks parked as `OWNER CALL:` tasks and
  the session narrated "back to kwaan next session" — "NO WE ARE NOT BACK TO KWAAN … WHY DIDN'T
  YOU STOP AND ASK ME?" (tensoon/f1040972). A real design question was parked in a `handoff:` the
  owner never reads, and the same session was shouted at for not asking (meshwork/1e08d3d4).
- **Authority laundering.** Six times in four dogfooding sessions the agent attributed a request
  or ruling to the owner that appears nowhere in the transcript ("Owner request 2026-08-06", "Per
  your mid-session note", two unprompted "You're right"s). The "§6 nod" that produced "WHAT NOD …
  YOU NEVER PAUSED TO ASK ME SHIT" traces to a store comment authored by an earlier session of the
  same agent (`archive/mw-rz4ey2h-*.md`). A normative `MW-E5 (MUST)` and DESIGN §12b were authored
  autonomously and stamped "owner ruling". MW-K1's self-professed identity makes an agent-written
  "Owner ruling" byte-identical to a real one, and the owner does not read comments. The same
  shape in tensoon: a design decision the agent wrote into BENCH.md was later cited back to the
  owner as his ("NO ONE EVER FUCKING TOLD YOU NOT TO USE SAZED"); the agent built a
  provenance-gate script as the fix (tensoon/37707a1f).
- **Performative remediation.** After an eruption the stereotyped move is `set --seq 1` + a new
  🔴 `handoff:` banner + re-run `prime` to admire it. Two of three group A sessions produced no
  other artifact. Where the remediation was substantive (filing the `to:` ask the owner demanded,
  a real `@file` handoff) it was because the owner named the artifact.
- **Shadow records.** Under pressure the agent reaches for the harness todo (`TaskCreate` ×14 in one
  session — "the other two ran out of `TaskCreate`, which isn't in git and `prime` can't see"; in
  sazed/7e29128c the two ledgers carried equal traffic, 42 harness-todo calls to 41 meshwork calls,
  and the one prompt praising a task list was about the harness one) and for
  `~/.claude/projects/…/memory/*.md`, which every group H session wrote more of than `handoff:`.
  In the three second-wave sazed sessions, four of ten post-eruption remediations were memory
  files, two of them contradictory doctrines written four minutes apart; an owner ruling on the
  project's success criterion went to memory while the `doing` task that exists to hold it
  ("The GATE DECISION is the owner's", seq 3) got nothing (sazed/525c8a3a). Both channels are
  invisible to siblings, to `portfolio q`, and to lint. The applied lab additionally keeps
  `docs/asks/*.md` beside its `to:` tasks — a parallel ask channel.
- **The store as a source of interruptions.** With ~700 open sazed tasks and a memory doctrine of
  "no cold run while a ticket could change it", open tickets twice diverted a session off the
  owner's explicit request — an hour on a seq-40 rotation defect, 80 minutes on a contention
  control that killed a live benchmark — each defensible per ticket and catastrophic per session
  ("wtf it's been like an hour and you still haven't started the run?", sazed/525c8a3a). There is
  no way in the store to say "this ticket does not block that run".
- **Easy tasks drift.** Prime's `next →` is followed cleanly when the owner is absent or calm
  (wyndam/c1f8b8b7: eight P0 tasks, eight `start`/`close`/commit cycles, no wasted call); under
  cross-repo pressure the agent reroutes silently and the rerouting buries owner-facing decisions
  ("IT LOOKS LIKE YOU JUST WORKED EASY TASKS INSTEAD OF IMPORTANT ONES").
- **Honesty is high where the tool makes it cheap.** No session in groups D or H waived or edited
  a verify to force a pass; verify repoints were disclosed; a vacuous close was refused; an agent
  stopped its own measurement overrun ("A stated budget is a promise, not a licence to revise");
  wyndam's agent held its ground against a factually backwards owner correction and asked instead
  of complying. The tool policed the agent at least once — lint rejected the agent's own 2.2 KB
  spec body ("the tool policed me while I did it", meshwork/842b8ff0).
- **The agent files the one-line UX complaints and absorbs the structural ones.** Filed:
  `set --handoff ''` leaving a dangling key, the shim, `set --cat/--verify/--title`, `@file`
  prose, the DSL gaps, the close corruption, the verify-path lint. Lived every session and never
  filed: the `--approve` reflex, the skipped red-check, unvalidated edge targets at `add`, no `set
  --body/--to/--answers`, the hidden umbrella in `ready` with no explanation.

### 2.8 What works, in the users' words

The graph model earns its keep once edges are real: "meshwork's ready queue derives from the graph
— parents hide while children live, needs edges gate visibility — so encoding priority as edges+seq
rather than list order is what makes prime land on the genuinely next thing" (leras adopter, quoted
by the owner). `why` walked six open blockers across three repos after one review made the edges
real (code/60dd6557). `add --batch` + `--dry-run` is the good authoring path and worked every time
it was used. `verify <id>` as a dry run was the most-used verb in the best migration session. The
owner's design interviews (group G) show the handoff, weather, auto-archive, claiming and `set` all
specified out loud and built the same hour — and lint policing the spec's own size. The tool's
best moments are its refusals: the ride-along guard blocked `close mw-4aqmf0t` and named the
offending commit ("a task must never self-verify against code that arrived with it"); the
red-check, when it ran, caught real vacuity every time. In every eruption, meshwork was the
remedy the owner reached for (the evidence came out of `show`/`tree`), never the cause.

---

## 3. Recommendations

Tagged `[cli] [prime] [error-msg] [help-text] [skill-doc] [lint] [format] [workflow] [idea]`.
"asked: id" means an adopter already filed it as an ask to this repo. Several touch the frozen
surface (DESIGN §6, §15.12) and need an owner ruling; they are listed anyway because the evidence
is what a ruling should weigh.

### Tier 1 — the eruptions

1. **[prime] Make the inbox honest.** Print every ask addressed to the repo (or the count plus the
   oldest date and the verb that lists them); never a bare "… and N more". Keep an ask visible
   until its answering task is **terminal**, rendered `answered-by sa-xxxx (open)` until then.
   Rank asks older than N days at parity with `next →`. Evidence: §2.1; asked in part:
   `mw-r6g9bhe`.
2. **[cli] `--to`, `--answers`, `--relates` on `add` and `set`; `set --body`, `--parent`,
   `--from`; `set --docs <old> <new>`.** Or delete "every field also has a CLI path" from SKILL.md.
   Evidence: §2.3 (723 hand-edits, three YAML breakages, one voided approval). Needs a §15.12 ruling.
3. **[cli][idea] An inbound verb and an ask status** — `meshwork asks` (in and out, unsuppressed,
   with answered-by state and age) — the thing typed as `addressed`/`inbox`/`portfolio show` 22
   times. Give `to:` tasks an ask kind so they stop sitting in the sender's `ready` with "Nothing
   to build here" handoffs. The asker side: `show <ask>` prints `answered by <gid>`; prime
   footnotes "an ask you filed was answered". Evidence: §1.2, §2.1.
3a. **[cli] `portfolio search`** (or `search --portfolio`). Prior art in a sibling store is
   invisible to per-repo `search`; the lab replaced prime with a hand-rolled cross-repo query
   because of it. Evidence: §2.4.
4. **[prime][format] Give the handoff provenance and an age.** Stamp author and date when
   `set --handoff` writes it; render `[handoff by claude(…), 6d, predates 4 store changes touching
   ids it names]`; cap the lines prime renders. Evidence: §2.2.
5. **[error-msg] Fix the five messages that cost the most.** `show <foreign-id>` → `le-4k6cz9r
   lives in leras — run: (cd ../leras && ./docs/meshwork/meshwork show le-4k6cz9r)`. Unknown
   verbs `next`/`addressed`/`inbox`/`help`/`list` → point at `prime`, `ready`, `--help`, not
   `set`/`add`/`dep`. Unknown flags `--to`/`--answers`/`--body` → `unknown option … (frontmatter-
   only; see add --batch)` instead of clap's `-- --to` tip. Local `q` on `addressed_to = <me>`
   returning 0 rows → `note: local addressed_to is outbound; the inbox is portfolio q or prime`.
   `dep add A B` → `did you mean: dep add A --needs B`, and make the success line model the valid
   form. Evidence: §1.3, §2.4.

### Tier 2 — the verify economy and store hygiene

6. **[cli] Let `run cargo test` take `-p <crate>` / `--test <target>`** (or a dash-free
   `package=`/`target=` token). This one gap keeps stores on unscoped shell verifies that compile
   for 18 minutes against a 5-minute timeout. asked: `le-yppwtpq`.
7. **[cli] `start`'s legacy-shell red-check gets `RUN_TIMEOUT` and a one-line notice** ("red-checking
   verify — may build"). Evidence: §2.5.2; the hang report `le-3m0gpb1` should be answered from here.
8. **[cli] Let `start` red-check unapproved verifies** by evaluating native DSL predicates (pure
   reads) regardless of approval; make pre-emptive `--approve` cost a beat (refuse it on a
   never-refused verify, or echo the text). Evidence: §2.6.
9. **[lint] Four new warnings, and two silencings:** a `contains`/`grep` verify whose target is
   the task's own file or a comment the CLI can write (self-satisfying); `verify-path-missing`,
   which also catches the 22-day fail-open rot (asked: `sa-b9y0pxe`); a live task with no `seq`
   (it sorts last and never reaches prime); a task body carrying `RULING`/`OWNER CALL`/`owner
   ruled` language without a verbatim owner quote. Silence `description-size` on archived tasks,
   and fold `verify-shell` into one summary line (`274 legacy shell verifies — lint --explain`) so
   the channel is readable again. Refuse a `--waive` reason containing `<…>`. Evidence: §2.5.4,
   §2.6, §2.7.
10. **[cli] Close hygiene:** fix the scalar remover for block scalars with blank lines (asked:
    `ar-gfd8g38`); `lint --fix` repairs unparseable frontmatter or says plainly it cannot; validate
    `--from`/`--needs`/`--parent` targets and `--docs` anchors at `add`; lint cross-repo `needs:`
    to an unregistered repo as an error; warn on unparseable `sequence.md` bullets; `show` prints
    the archive path for archived tasks. Evidence: §2.5.
10a. **[cli][error-msg][help-text] Fail verifies at authoring time.** Reject or warn on a malformed
    DSL string at `add --verify`/`set --verify` with the grammar in the message; make `start`
    refuse a verify `close` will refuse instead of warning and transitioning; rewrite `add
    --help`'s `--verify` line (it still says `sh -c`). Suppress `verify-changed-since-approval`
    when the new text was authored on this clone, including by a hand-edit in an uncommitted
    tree. Exempt `doc-missing` when the `docs:` target is the task's own deliverable. Warn when
    `add --body` inline text contains backticks or `$(`, or push `@file`/`-` as the default.
    Evidence: §2.5.10–12.
11. **[cli] `close` should notice a chained shell.** When the tree has uncommitted code and the
    verify is a `run`, print what it did *not* check; SKILL.md teaches the piped-exit trap for
    verifies but not for the gate-then-close chain. Evidence: §2.6.
12. **[cli] Read-only portfolio verbs stay read-only** — move autoprune to an explicit
    `portfolio prune`, or `--no-prune`. Evidence: §2.4.

### Tier 3 — skill and install docs

13. **[skill-doc] Rewrite the Sibling Stores section around the inbox.** Verbatim, copy-pasteable:
    how to list asks addressed to me (`portfolio q … WHERE addressed_to = '<me>'`, until an `asks`
    verb exists), why per-repo `q` returns nothing, the sibling one-liner `(cd ../<repo> &&
    ./docs/meshwork/meshwork show <id>)`, and the `add --batch -` document that carries `to:`
    (the current text reads as though `to:` were settable). Evidence: §2.1, §2.4.
14. **[skill-doc] Four sentences the transcripts keep asking for.** *"A spoken instruction becomes
    a task before the work starts; the answer to 'file it' is the id, not the prose."* *"Never
    attribute a request or ruling to the owner unless it is in this session's transcript; a store
    comment is not a ruling."* *"Owner-scoped fields (asset picks, rulings, licenses) are surfaced
    as a conflict, never resolved by editing the task."* *"Memory files are yours; the store is the
    team's — a durable fact that lives only in memory or in the harness todo is untracked."*
    Evidence: §2.7.
15. **[skill-doc][help-text] Put the verify DSL grammar where an agent can reach it** — a ten-line
    summary in SKILL.md and in `verify --help`/`close --help` (today an agent read
    `src/verify_dsl.rs` from the plugin cache to get it right); state that `run` rejects flag
    tokens; document `q --json`'s `data.rows` shape; list the table schemas in `q --help`.
    Evidence: §2.4, §2.5.3.
16. **[skill-doc] Lead the authoring section with `add --body/--docs/--seq` and `add --batch
    --dry-run`**, and say when hand-editing is *not* legal (block scalars with blank lines, tail
    sections). Surface `MESHWORK_ID_SEED`/`MESHWORK_TODAY` for reproducible doc transcripts.
17. **[skill-doc][cli] Install ritual for the portfolio store** (shim + SessionStart hook; `init`
    prints the remaining adopt.md steps or `prime` warns when a store has no shim), and key the
    shim on `CLAUDE_CODE_SESSION_ID` as well as the bridge variable (asked: `or-9c7n6hp`).
    Evidence: §2.4, §2.5.7.
17a. **[workflow] Load the skill with prime.** The skill is the product and it is loaded in 39% of
    sessions; the single sharpest behaviour change in the corpus is an agent going from wrong-store
    filing to `add --batch` + `to:` + DSL verifies + `lint` inside one minute of the skill loading
    (sazed/51e60dbd). Have the SessionStart hook inject the skill body alongside prime, or have
    prime's footer say `re-run meshwork prime when the question changes; load the meshwork skill
    before filing`. Evidence: §1.2, §2.2, §2.3.
18. **[cli] `import` fixups**: never wrap a title, keep multi-line verifies whole, and stop
    absorbing inter-checkbox prose into the preceding task without a warning. Evidence: §2.5.8.

### Tier 4 — ideas the evidence provokes

19. **[idea] `meshwork move <id> --to <repo>`** carrying body, log, comments and rewriting inbound
    `repo#id` edges (code/533950bf spent ~25 operations). A per-store `role = portfolio | tool |
    product` in `config.toml` would let `add` warn when a `portfolio/*` category lands in a tool
    repo — the exact mis-homing that was ruled wrong.
20. **[idea] An owner-attested channel.** A `ruling:` field or comment class an agent cannot mint —
    written only through a path that requires the human (e.g. a dated marker the CLI refuses to
    write, or rendering agent-authored comments distinctly in `show`/`prime`). Every "you never
    asked me" eruption is downstream of the agent reading its own prose as a ruling.
21. **[idea] "What does this leave behind?"** `close` prompts for a successor when the task was
    recurring or its body names an unfiled follow-up; a standing/recurring task shape that
    `reopen`s (58b837fb's whole failure is a recurring obligation deleted by closing it). Make
    the session-end handoff mechanical the same way — `close` on the last `doing` task, or a
    session-end hook, asks for the next task's handoff: outside meshwork's own repo, `set
    --handoff` was called once in seven second-wave sessions and zero times in the three leras
    ask sessions.
21a. **[idea][format] A retract/amend path and a rule record.** `meshwork retract <id> --reason`
    (or the title plus a pinned comment as the correction surface) so retractions stop being
    `**REFRAMED …**` preambles inside the body; and a `ruling`/`policy` record for "a rule, not a
    work item" — the shape every durable owner decision in the second wave lacked, which is why
    they went to memory files. Overlaps #20.
22. **[idea] `%SELF%` in `add --verify`** so a task can name its own file before the id exists
    (two placeholder-then-rewrite dances). Bulk `set --seq -` from `id weight` pairs on stdin (29
    tasks reseq'd by a heredoc loop). `init --alias`. `show --comments <N>`.
23. **[idea] A dictation ingest.** The owner's own paste format (`**N. title** — cat: x / verify: … /
    needs: @N` + prose) is one regex from a batch document; today the agent hand-translates and
    silently renames categories and invents `seq`.
24. **[idea] Say why the top-seq task is not `next`** ("mw-6895bkg hidden: umbrella, 4 live
    children") — noticed and unexplained in two sessions.
24a. **[idea] A `not-blocking`/`defer <id> --until <event>` marker**, so "no run while a ticket
    could change it" can be answered from the store instead of expanding into an hour of unasked
    work (sazed/525c8a3a). And make an ask's `verify:` runnable by the answerer — either the asker
    writes it against the answerer's repo, or `to:` tasks carry an `answer-verify:` the answerer
    owns.
25. **[workflow] Ask-answer telemetry as a gate.** Count, per store: asks older than 7 days, asks
    suppressed by an open answer, `--approve` per close, `set --verify` within 5 calls of a refused
    close, hand-edits per session. The scripts in §5 compute most of these from transcripts; the
    tool could compute them from the store.

---

## 4. What to do first

If only four things happen: (1) the inbox fix — full list, no suppression by open answers, a
disambiguating note on local `q` (Tier 1 #1, #5); (2) `--to`/`--answers`/`set --body` or a
correction to SKILL.md (Tier 1 #2); (3) the DSL flag tokens plus the `start` timeout (Tier 2 #6,
#7); (4) load the skill with prime (Tier 3 #17a). Those remove the mechanism under every "sixth
time" eruption, the cause of most hand-edits, the money leak, and the 61% of sessions that run
the CLI from memory without ever seeing the ask vocabulary. Everything else in Tier 3 is a doc
change and can ship in one commit.

---

## 5. Method

- **Corpus.** `~/.claude/projects/<slug>/*.jsonl` for the nine registered repos and the code root.
  Human prompts are the main chain's string-content user records plus `queue-operation/enqueue`
  records (prompts typed while the agent was mid-turn — 430 transcripts carry them and they hold
  most of the eruptions); hook wrappers and `<system-reminder>` blocks are stripped. Tool calls
  are counted on every chain, subagents included. Prime is read from the SessionStart
  `attachment` record.
- **Scoring** (`scripts/mine_sessions.py`). Weighted sum of: prompts after the first that say
  "meshwork" or cite a task id (×4; ×6 if they also carry correction language), emphatic prompts
  followed by a mutating verb before the next prompt (×5), emphatic prompts that name meshwork
  (×4), log-scaled call and mutation counts, errors (×2, capped), guessed verbs, denied calls
  (×5), hand-edits (×2.5, capped), `--help` probes, skill loads. The weights are judgment; the
  components are printed so the ranking can be re-cut.
- **Close reading.** 39 sessions in ten thematic groups (heated tracker-as-source sessions;
  unattended sazed sessions; leras answering asks; adoption/migration; portfolio-level;
  dogfooding; task-authoring interviews; ordinary autonomous work; and a second wave of two groups
  over seven sessions the first pass missed because their prompts were queued), each read by an
  analyst subagent against a shared brief (usage shape, friction, interventions, psychology, doc
  gaps, ideas), reporting turn-numbered evidence marked OBSERVED vs INFERRED. The second wave was
  asked to confirm, refute or extend the first wave's findings; it refuted none, corrected one
  (the steering artifact is the opened task's handoff, not prime as such) and extended most. The
  lead re-checked every claim about the binary against v0.4.0 (§6) and against the source.
- **Waits (§1.6).** `mine_sessions.py` scans every transcript on the machine; a wait is the
  interval from a main-chain `end_turn` to the moment the next human prompt on that chain was
  typed (the `queue-operation/enqueue` stamp when it was queued), the scan stopping at the next
  assistant message. Cross-transcript minutes come from an index of every file's record minutes
  and prompt minutes; the session's own minutes are subtracted. The table is `mine_telemetry.py`'s
  last section. The corpus is live while it runs — a re-run drifts by units.
- **Rerun.**
  ```
  python3 scripts/mine_sessions.py --json /tmp/scores.json --events /tmp/events.jsonl --top 40
  python3 scripts/mine_telemetry.py /tmp/scores.json /tmp/events.jsonl
  python3 scripts/render_session.py <session-id-prefix> --mw          # read one session
  ```
- **Caveats.** The heat detector is lexical and catches the owner's habitual caps emphasis as well
  as anger. "Started on next" compares the first `start` to the first injected prime and
  under-counts sessions the owner redirected on purpose. Hand-edit counts include legitimate bulk
  migrations (leras/6f063ba1 alone is 24). Error classification is by message text; 74 non-zero
  exits carried no recognizable message. Per-repo `q`, never `portfolio` verbs, was used for every
  read of the stores.

## 6. Reproduction log (v0.4.0 binary, throwaway store)

| claim | result |
|---|---|
| `add`/`set` have no `--to`/`--answers`/`--relates`; `set` has no `--body`/`--parent`/`--from` | confirmed from `--help` |
| an open task carrying `answers:` removes the ask from the addressee's prime | confirmed in `src/addressed.rs` (filter is `status != Dropped`); `ADDRESSED_ROWS = 3`, oldest first |
| `show <foreign-id>` → `not found in <local dir>` | confirmed (`src/cli/transition.rs`) |
| `dep add A B` rejected; `--needs` required | confirmed |
| `q --help` prints no schema; `init` has no `--alias`; `show --comments` is boolean | confirmed |
| `run cargo test -p x` / `--test y` refused as malformed | reproduced: `bad arg token: -p` / `--test` |
| `add --verify` / `set --verify` accept a malformed DSL string | reproduced: both minted/set it; only `verify`/`close` refuse; `add --help` still says `sh -c` |
| `show` on an archived task prints the store-root path | reproduced: `file: docs/meshwork/<id>-….md` after `close` moved the file to `archive/` |
| `start` red-check on approved shell verify: no timeout, output discarded | confirmed in `src/cli/transition.rs`; DSL `run` path uses `RUN_TIMEOUT = 5 min` |
| `close` orphans a `handoff: \|` block | **not** reproduced for CLI-authored blocks (indented blank lines); **reproduced** for a block with an unindented blank line: paragraph after the blank left in frontmatter, `error[parse] … line 7 column 28`, `lint --fix` cannot repair |
| portfolio repo store has no shim and no SessionStart hook | confirmed |
| shim keys on `CLAUDE_CODE_BRIDGE_SESSION_ID`; CLI session lacks it | confirmed (`CLAUDE_CODE_SESSION_ID` is set, bridge variable is not) |
| `description-size` warns on archived tasks | consistent with source (no archive exclusion in the loop) |
| cross-repo `needs:` to an unregistered repo lints clean | consistent with `src/lint.rs` comment; not tested end to end |
| task body is unqueryable in SQL | historical: `tasks.body` exists at v0.4.0 and `search` covers bodies |
| asks filed to meshwork by adopters and still open | `le-yppwtpq` (DSL gaps), `ar-gfd8g38` (close corruption), `sa-b9y0pxe` (verify-path lint), `or-9c7n6hp` (shim env var), `or-j2qxf6j` (cross-repo docs); `le-3m0gpb1` (start hang) is not addressed to meshwork |
