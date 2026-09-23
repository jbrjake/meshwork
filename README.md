# meshwork

**for when you make a real mesh out of things**

An opinionated, minimalist todo list for clankers. Tasks get their own human-readable markdown files logged in git. There's a Rust CLI to manage them. There's no database: the CLI runs SQL queries directly against the markdown. And it can materialize them into a token-conscious session-start digest to keep agents on track.

It's a mesh twice over: because tasks are modeled as a graph with edges to related tasks, and also because the git repos federate. A task in one project can depend on a task in another project, or even request new work from another, and you can set up a portfolio view spanning multiple repos.

## install

```
/plugin marketplace add jbrjake/claude-plugin-marketplace
/plugin install meshwork@jbrjake
```

That installs the skill into Claude Code. Ask a session to "adopt meshwork in this repo" and it handles the rest ([getting it](#getting-it) has the manual path).

## quick-start

```bash
$ meshwork init

$ meshwork add "Do the thing with the stuff" --cat stuff/doodads --verify "run cargo test stuff::thing"
ac-acnxdkg
  docs/meshwork/ac-acnxdkg-do-the-thing-with-the-stuff.md

$ meshwork prime
acme — 1 open
store @ c601195 · 1 uncommitted task edit
stuff/doodads 1
weather:
- flow 7d: filed 1 (0 from other tasks) · done 0 · dropped 0 · backlog 1 (+1)
- queue: ready 1 of 1 open · doing 0 (0 stale) · blocked 0 · open age med 0.0d · 0 past 14d
next → ac-acnxdkg Do the thing with the stuff
  [stuff/doodads]
  verify: run cargo test stuff::thing
rules: an instruction becomes a task before the work starts (the answer is an id) · an ask is a to: line in your own store
owner-scoped fields raise a conflict, never an edit · a ruling counts only from this transcript · re-run prime when the question changes; load the meshwork skill before filing

$ meshwork start ac-acnxdkg --as claude # irl it uses a shim to pull in session id
note: red-checking ac-acnxdkg's verify — may build; up to 300s
ac-acnxdkg open→doing
$ meshwork comment ac-acnxdkg --as claude "Smoking gun! You're absolutely right. This seam is load-bearing. On it. Cerebrating..."
ac-acnxdkg: comment added as [claude]
...
$ meshwork close ac-acnxdkg
note: ac-acnxdkg closes against an uncommitted tree — this close checked nothing but its verify on: src/lib.rs, Cargo.lock, docs/
ac-acnxdkg doing→done (verify exit 0)

$ meshwork q "SELECT category, count(*) AS n FROM tasks WHERE status='done' GROUP BY category ORDER BY n DESC"
category | n
stuff/doodads | 1
(1 rows)

$ cat docs/meshwork/archive/ac-acnxdkg-do-the-thing-with-the-stuff.md
---
id: ac-acnxdkg
title: Do the thing with the stuff
status: done
category: stuff/doodads
verify: run cargo test stuff::thing
created: 2026-08-12T21:28Z
---

## log
- 2026-08-12T21:28Z created
- 2026-08-12T21:29Z open→doing — claimed by claude
- 2026-08-12T21:29Z doing→done — verify exit 0 @ c601195+5

## comments
- 2026-08-12T21:29Z [claude] Smoking gun! You're absolutely right. This seam is load-bearing. On it. Cerebrating...
```

*`./scripts/demo.sh` plays that whole loop on a throwaway scratch repo. It doesn't hit the network and it cleans up after itself.*

## why?

If you code with agents for more than toy projects, you manage your context window size rather than rely on compaction. You have to make sure new agents get on-boarded at session start and kept on-task.

### you may not need this

You can get a lot done just with checkbox lists in markdown files like TODO.md, a HANDOFF.md to relay session context to fresh agents, and a project rule to 'pause at natural breaking points and persist relevant context to disk to handoff to a new session'. When it wraps up, you run `/clear`, then type 'Read the handoff and get up to speed, then continue building the project, committing and documenting as you go.'

### until you do

Over time, though, you develop a lot of handoff context:

* Some tasks don't fully complete. Maybe you need to wait for a test box to become available for a benchmark, or you're blocked on a bug in another project, or you just switch focus to something shinier.
* Eventually you end up with multiple in-flight threads of work and the agent needs to be apprised of the state of all of them.
* Agents sometimes forget to mark items complete, or remove completed items.
* Agents are just...chatty. They will sometimes go into exhaustive detail on stuff no one cares about.

It becomes a problem because of the context hit. These files get read in all the time. Extraneous stuff costs tokens and focus. So you add a rule to the project to rotate old items to an archive.

Of course, agents aren't reliable at following rules. So you add a hook with a deterministic trigger: you cannot commit code if your TODO.md or HANDOFF.md are above N lines.

Of course, they'll game that and have super long lines, so you change to characters. Either way, it's a game to them and they play golf trying to get under-but-as-close-to the limit as possible.

Of course, they're terrible at counting. So it can take them like 3-4 tries to cut the text down to the limit you set. Eventually it gets so time-consuming and token-expensive that you raise the ceiling when important stuff has to get tracked.

## numbers

### without meshwork

*Measured 2026-08-04, across the history of two working repos.*

| | Project A | Project B |
|---|---|---|
| Edit calls targeting TODO.md + HANDOFF.md | ~20% (927 of 4,693) | ~16% (641) |
| sessions reading both files within their first 5 tool calls | ~95%, at a ~22K-token tax each | ~90% |
| worst line-cap thrash episode | 34% of a session's shell calls (14 gate failures in 84 minutes) | 26 consecutive tool calls |
| line limit increases | 500→550 | 200→750 |

One repo's CLAUDE.md proudly declared its worklist was "131 lines." It was 38KB.

### with meshwork

*Measured after both repos migrated to meshwork (Project A 2026-08-07, Project B 2026-08-10). Each table shows a repo's last 10 sessions before meshwork vs its working sessions after migrating.*

Every turn repeats the entire conversation, so the first things in context get repeated the most. Project A's pre-migration sessions opened by reading ~28K tokens of TODO.md + HANDOFF.md. Reran on all 153 following requests, that read compounds to ~4M tokens. And that's before you account for all the TODO and HANDOFF edits the agent makes during the session. Altogether it's 4.19M tokens.

Four million accrued busywork tokens across turns is a drop in the bucket for any serious coding session. This isn't about cost savings. It's about the time lost to all those turns while the agent thrashes against a todo list in markdown, and the lost focus of bringing extraneous content into the context window. Interestingly, after migration to meshwork, sessions seem to work more efficiently. Project A sessions carry 90K → 113K real work tokens/session and run ~28% longer (154 → 197 useful requests/session).

**Project A** (Claude Opus sessions):

| | before | after (34 sessions) |
|---|---|---|
| session-start onboarding read | tool calls to read TODO.md + HANDOFF.md: **116,119 bytes** (~28K tokens) | SessionStart-injected `meshwork prime`: **3,762 bytes** (~940 tokens) **31× less** |
| busywork, counted once | **~32.6K tokens/session**, 26.6% of session content, 1 busywork token per 2.8 of work | **~8.5K**, 7.0%, 1 per 13.3, **3.8× less** |
| **busywork, compounded** | **4.19M tokens/session**, 10.6% of the 39.7M the session re-submits over ~154 requests | **0.99M**, 2.1% of 48.3M over ~197 requests, **4.2× less** |
| worklist fidelity | one 550-line TODO.md (124 checkbox entries, ~4 lines each) | 224 tasks, 40 dependency edges |

**Project B** (Claude Fable sessions):

| | before | after (3 sessions) |
|---|---|---|
| session-start onboarding read | tool calls to read TODO.md + docs/HANDOFF.md: **96,155 bytes** (~24K tokens) | SessionStart-injected `meshwork prime`: **4,023 bytes** (~1K tokens) **24× less** |
| busywork, counted once | **~31.0K tokens/session**, 30.1% of session content, 1 busywork token per 2.3 of work | **~10.0K**, 10.5%, 1 per 8.5, **3.1× less** |
| **busywork, compounded** | **2.34M tokens/session**, 9.5% of the 24.7M the session re-submits over ~105 requests | **0.50M**, 2.8% of 18.1M over ~96 requests, **4.7× less** |
| worklist fidelity | a 733-line TODO.md + a 666-line HANDOFF.md (48 checkbox entries, hard-wrapped ~12 lines each) | 68 tasks, 63 dependency edges |

*See `scripts/busywork-tokens.py` for the math.*

## when meshwork makes sense

| reach for meshwork when… | reach for something else when… |
| --- | --- |
| the work happens in **many short agent sessions** that must hand off cleanly | one long-lived human holds the context in their head |
| tasks have a **runnable definition of done** (`verify:` a test, a grep, a file) | done-ness is a conversation |
| **git is the source of truth** and work must survive offline, in worktrees, across machines | you need webhooks, boards, assignees, notifications |
| one owner, a portfolio of repos | a team that needs permissions and a web UI |
| you want the tracker's own output **capped in bytes**, because context is the scarce resource | context is 'free' because a human is reading |

## the command line workflow

*Every terminal transcript below is pasted from a real run of the binary.*

### bootstrapping

Adopt in an existing repo with `meshwork init`.

Or migrate an existing TODO.md with `meshwork import todo TODO.md`, and each checkbox becomes a task file. Nested checkboxes get parent/child relationships.

### adding tasks

File work as tasks with dependencies and a runnable definition of done:

```
$ meshwork add "Reproduce the 600M-row spill cliff" --cat engine/spill --verify "exists repro.log"
sa-nmvpyqr
  docs/meshwork/sa-nmvpyqr-reproduce-the-600m-row-spill-cliff.md

$ cat docs/meshwork/sa-nmvpyqr-reproduce-the-600m-row-spill-cliff.md
---
id: sa-nmvpyqr
title: Reproduce the 600M-row spill cliff
status: open
category: engine/spill
verify: exists repro.log
created: 2026-08-06T21:47Z
---

## log
- 2026-08-06T21:47Z created
```

We can also add tasks with dependencies and an explicit `seq` (the priority weight). And note how categories are hierarchical, with slashes separating the levels:

```
$ meshwork add "Fix spill batch sizing" --cat engine/spill --needs sa-nmvpyqr --seq 10 --verify "run cargo test spill::batch"
sa-38wd6se
  docs/meshwork/sa-38wd6se-fix-spill-batch-sizing.md
$ meshwork add "Write the spill postmortem" --cat docs --verify "exists docs/postmortem.md"
sa-jt7zg9w
  docs/meshwork/sa-jt7zg9w-write-the-spill-postmortem.md
```

You can draft a task without `--verify`, but one's got to exist in a failing state to start it, and it then has to pass to complete it. If it started already green you couldn't tell when the work is done, making it vacuous.

#### batching

Filing a whole interlinked batch at once is `add --batch`. You provide a stream of task documents on stdin, you set `@handle` refs between siblings that don't have ids yet, and all files get written or none do. Each document is the same frontmatter a task file carries, minus the `id:` that meshwork will add:

```
$ meshwork add --batch - <<'EOF'
---
handle: bench
title: Benchmark spill at 64k-1M batch sizes
category: engine/spill
needs: [sa-38wd6se]
verify: exists bench/spill.csv
---
Numbers before and after the sizing fix.
---
title: Tune the governor wakeup default
category: engine/governor
needs: [@bench]
verify: run cargo test governor::wakeup_default
---
EOF
sa-bbds8pt
  docs/meshwork/sa-bbds8pt-benchmark-spill-at-64k-1m-batch-sizes.md
sa-8w8m842
  docs/meshwork/sa-8w8m842-tune-the-governor-wakeup-default.md
```

The first document names itself `bench` as a local handle; the second depends on it through `@bench` before either has an id. On disk the handle is gone, rewritten to the minted id:

```
$ grep needs: docs/meshwork/sa-8w8m842-tune-the-governor-wakeup-default.md
needs: [sa-bbds8pt]
```

### exploring tasks

Then you can ask it what's actionable:

```
$ meshwork ready
sa-nmvpyqr  Reproduce the 600M-row spill cliff
sa-jt7zg9w  Write the spill postmortem
```

The blocked tasks don't appear, and you can ask why:

```
$ meshwork why sa-38wd6se
sa-38wd6se blocked by 1:
- sa-nmvpyqr (open) — verify: exists repro.log
```

`lint` is aware of the task graph. That fix is ranked `seq: 10`, but depends on another task that isn't ranked, so it lets you know you're deadlocked:

```
$ meshwork lint
warning[needs-behind] sa-nmvpyqr: unranked, behind sa-38wd6se at seq 10 which needs it — report only; the fix is a rank for this one or a park for that one, and only the owner knows which
0 error(s), 1 warning(s)
```

### closing tasks

meshwork tries to prevent closing tasks without doing the work:

```
$ meshwork close sa-jt7zg9w
meshwork: sa-jt7zg9w stays open: verify failed — exists docs/postmortem.md: no such path
```

As you can see, there are limits to enforcement. An agent absolutely will just touch the file to hit that requirement. meshwork isn't guaranteeing anything more than that the provided validation passes.

*`close --waive "reason"` exists for the genuinely unverifiable. It's recorded and queryable as `WHERE waived IS NOT NULL` so you can track it.*

#### task verification security

Tasks can be verified with a small expression language, or with shell execution and all the security risks that entails.

##### the verify dsl, the preferred path

A verify field that leads with a DSL keyword is parsed as a predicate instead of being handed to a shell:

```
verify: exists bench/spill.csv
verify: exists bench/spill-*.csv
verify: absent src/legacy_parser.rs
verify: contains CHANGELOG.md /^## v0\.5/
verify: lacks src/lib.rs legacy_parser
verify: run cargo test spill::batch
verify: run cargo test package=engine target=spill batch
verify: all(exists bench/spill.csv, run cargo test spill::batch)
```

`exists`, `absent`, `contains` (literal or `/regex/`, anchored per line like grep) and `lacks` (the inverse of `contains`) evaluate natively. No process runs, and there's nothing to approve, because they're just reads.

`run` executes a real command, but not through a shell: the arguments are validated against a very restricted per-runner grammar (today `cargo test`, `cargo build`, and `cargo fmt`) and spawned directly as an argument list, so shell metacharacters are just characters that fail to parse. No argument may lead with a dash. That means, rustaceans, that `package=<crate>` and `target=<name>` are how you spell `-p` and `--test`.

Nothing an author types ever becomes a flag. Paths are repo-relative and can't traverse out.

Because there's no shell to smuggle anything through, DSL verifies generally skip the approval ceremony below that shell execution uses. `run` stays approval-free only under certain conditions. If any commit ever delivered the task file alongside code or any other changes outside the meshwork tasks directory, it has to be approved like shell does. Tasks never vouch for code that arrived with them.

`run cargo test` also closes a classic hole: `cargo test` exits 0 when a filter matches nothing, so the DSL demands an observed `ok. N passed` with N ≥ 1 before it counts as green. A filter that matches zero tests can never close a task.

Text that doesn't lead with a keyword gets treated as plain shell, falls to the path below, and `lint` nags about it. Keyword-led text that doesn't parse is refused where you write it (`add`, `set`, a batch file) rather than falling back to shell, and `start` won't open work behind one.

#### shell execution

Task files can come from untrusted sources, like third-party PRs. `verify:` fields outside the DSL above are executed in the shell. This is not a fantastic combination for security.

When you add a task in meshwork and provide the verify field, it's trusted and will be run when you close a task. The assumption is that you or an agent you're delegating responsibility to is trusted. And if it's an agent, that you've configured your harness with the security you need.

If task files get added or modified outside the CLI, their verify fields aren't implicitly trusted. That includes when you edit the markdown files directly, or when you pull down changes from a remote.

Instead they're trust-on-first-use: `close` refuses them (`refusing unapproved verify`) until you or your agent approve the exact text of a task's verify field with `close --approve`. The approval is recorded per clone, and it's stored outside git, where a merge can't plant one.

*Reviewed checkouts — CI, gates — may grant `MESHWORK_TRUST=1` instead.*

What this means is the human in the loop is responsible for security. If you accept tasks from other people, make sure you read the contents of anything they will execute before you kick off an agent session. If you just hit your enter key to every Claude prompt, all bets are off.

### work loop

So let's get to work:

```
$ meshwork start sa-nmvpyqr --as claude
sa-nmvpyqr open→doing
$ meshwork comment sa-nmvpyqr --as claude "cliff reproduces at batch=64k; tracks the governor wakeup interval, not batch size"
sa-nmvpyqr: comment added as [claude]
$ touch repro.log        # stand-in for the actual work
$ meshwork close sa-nmvpyqr
sa-nmvpyqr doing→done (verify exit 0)
```

Before wrapping up a session, leave a note on whichever task is up next:

```
$ meshwork set sa-38wd6se --handoff "Cliff is governor wakeup, not batch size — don't burn a session re-deriving that (comment on sa-nmvpyqr has the repro). Try wakeup=250ms before touching batch math."
sa-38wd6se handoff set
```

(`set --handoff` and `comment`'s text argument also accept `@<file>` and `-` for stdin, so long notes don't have the hassle of multi-line shell quoting.)

### session priming

The benefit of working this way isn't any magical belief that your vibe-coded slop works. That's what testing is for. It's so the next session doesn't have to read files to catch up or waste time at the end of sessions rotating tasks in text files. It gets `meshwork prime` injected automatically by a SessionStart hook, which materializes the handoff from the store:

```
$ meshwork prime
demo — 4 open, 1 done
store @ 15563ae
engine/spill 2 · docs 1 · engine/governor 1
weather:
- flow 7d: filed 5 (0 from other tasks) · done 1 · dropped 0 · backlog 4 (+4)
- queue: ready 2 of 4 open · doing 0 (0 stale) · blocked 0 · open age med 0.0d · 0 past 14d
- graph: 1 lanes · unlocks most sa-38wd6se (2) · needs-behind 0 · blocked on foreign 0 · unresolved 0 · owed to others 0
- friction 7d: close attempts 1 · reopens 0 · blocks 0 · thrash 0 · handoffs citing closed tasks 1
next → sa-38wd6se Fix spill batch sizing
  » Cliff is governor wakeup, not batch size — don't burn a session
  » re-deriving that (comment on sa-nmvpyqr has the repro). Try wakeup=250ms
  » before touching batch math.
  [handoff by claude, 0d]
  cites 1 closed task (sa-nmvpyqr) — handoff may be stale
  [engine/spill] · blocks: sa-bbds8pt
  verify: run cargo test spill::batch
also ready (1 more, top 1):
- sa-jt7zg9w Write the spill postmortem
recently done:
- 2026-08-06T21:58Z sa-nmvpyqr Reproduce the 600M-row spill cliff
rules: an instruction becomes a task before the work starts (the answer is an id) · an ask is a to: line in your own store
owner-scoped fields raise a conflict, never an edit · a ruling counts only from this transcript · re-run prime when the question changes; load the meshwork skill before filing
```

Almost everything in that digest is derived from the task files: counts, the category rollup, the weather, what's next and why, what just finished. The `store @` line is derived too, from git. A session landing on a stale clone sees uncommitted task edits and drift from upstream up front instead of discovering them mid-work.

The exception is the `»` lines. That's the `handoff:` block. It lives on whichever task is up next. It's also the one thing in the digest someone (or something) wrote, so meshwork says who and how long ago, and flags when it names a task that has since closed. The example above shows what the warning looks like, with a handoff that's sat there for weeks. Linting warns if you leave a handoff on a task you close.

The `weather:` block is the store's vital signs, computed from the log lines. It tracks what got filed and finished this week, how much of the backlog is actually ready, which task unlocks the most work, and the friction (failed closes, reopens, thrash).

The `rules:` footer is fixed. It's the four session rules the skill teaches, riding in the binary so a session that never loaded the skill still sees them.

The digest is capped at 6KB ≈ 1.5K tokens versus the 22K-token ritual it replaces.

## it's file-based

A task is one markdown file. This is the entirety of `sa-38wd6se`, the blocked task from the session loop:

```markdown
---
id: sa-38wd6se
title: Fix spill batch sizing
status: open
category: engine/spill
needs: [sa-nmvpyqr]
verify: run cargo test spill::batch
seq: 10
created: 2026-08-06T21:48Z
handoff: |
  Cliff is governor wakeup, not batch size — don't burn a session
  re-deriving that (comment on sa-nmvpyqr has the repro). Try wakeup=250ms
  before touching batch math.
---

## log
- 2026-08-06T21:48Z created
- 2026-08-06T21:59Z handoff by claude
```

Hand-editing is legal and expected but never necessary. Fields can all be set by the CLI with `meshwork add` at creation and `meshwork set <id>` after.

`meshwork lint` validates fields (schema, cycles, dangling edges, post-merge damage), and `meshwork lint --fix` repairs what it can. Since it sees the whole graph, it also flags what no single file can show: the `needs-behind` above, a handoff that cites closed work, two live tasks on one `seq`, or a verify that reads a file the tree doesn't have. Bulk findings fold to one line each in the text report, and `--explain <code>` unfolds them. A file that fails to parse isn't dropped. It shows up as an `invalid` row in every listing until someone fixes it.

The format has a spec, [FORMAT.md](FORMAT.md), that's versioned and self-contained. Anyone can implement against it without having to use this project's code.

When a task reaches `done` or `dropped`, to de-clutter, its file moves to `docs/meshwork/archive/` automatically (and moves back on `reopen`). Archived tasks stay loaded and queryable. Dependency resolution, SQL, and the digest are location-blind. Once a hundred archived files have piled up, `lint` says so and `lint --fix` concatenates them into a few bundle files. Once bundled, tasks still `show`, can be queried, and split back out upon `reopen`.

Because tasks are files in git, concurrency is git's problem. Two sessions in separate worktrees can create tasks, comment on the *same* task, and close tasks, then merge without manual conflict resolution. The one merge artifact git can produce (a duplicated frontmatter key from union-merge) is repaired by `lint --fix`. Tasks record when they're claimed by someone as active work, but it's not enforced.

The `seq` field is the priority: integers with gaps of 10, lower runs sooner.

## but you can query it like a database

SQL lets you ask questions:

```
$ meshwork q "SELECT category, count(*) AS n FROM tasks WHERE status='open' GROUP BY category ORDER BY n DESC"
category | n
engine/spill | 2
engine/governor | 1
docs | 1
(3 rows)
```

The `## log` lines are a table too, with every status transition timestamped. On top of the six raw tables sits a derived layer of thirteen views that's computed from the files on every query. `spans` is a row per stint in a state, so 'how long was that blocked?' is a `SELECT`. `facts` is a row per task with its queue, service and cycle hours. `graph` knows the structure:

```
$ meshwork q "SELECT id, unlock, depth FROM graph WHERE unlock > 0 ORDER BY unlock DESC"
id | unlock | depth
sa-38wd6se | 2 | 2
sa-bbds8pt | 1 | 1
(2 rows)
```

`unlock` is how many live tasks transitively wait on that one, which is what prime's `unlocks most` reads.

`stats` renders the whole layer as a report: the weekly flow, a close-hazard table saying what fraction of tasks survive each day unclosed, spans by state, the lanes of connected work, and the ten tasks the rest of the store points at most. `prime`'s weather is the same numbers boiled down to a handful of lines. The views are specified in [FORMAT.md](FORMAT.md) next to the file format and pinned by a conformance corpus, so a third-party reader can implement them. `q --help` lists every table and view with its columns.

The CLI also has a `--json` output flag for scripts and agents.

## pin tasks to the spec they implement

A task that cites the paragraph of a specification it implements introduces a problem. How can meshwork know if the spec changes? If you end a spec doc's headings with `{#sp-<slug>}`, the content in them can be treated as clauses with stable ids. `cover` pins the clause to a task, recording a hash of the text under the heading as it reads right now. The claim is "I implement this clause as it read when I said so":

```
$ cat > docs/SPEC.md <<'EOF'
## Spill batches {#sp-spill-batch}
Spill batches are 64k rows.
EOF
$ meshwork cover sa-38wd6se docs/SPEC.md#sp-spill-batch # cited as written
sa-38wd6se covers docs/SPEC.md#sp-spill-batch @cb5d4612ef80
$ cat > docs/SPEC.md <<'EOF'
## Spill batches {#sp-spill-batch}
Spill batches are 256k rows. # uh-oh, requirements change!
EOF
$ meshwork spec audit docs/SPEC.md # but meshwork can see the hash is stale
docs/SPEC.md: 1 clauses, 1 pins across 1 tasks
unclaimed (0): none
orphaned (0): none
stale (1): demo#sa-38wd6se docs/SPEC.md#sp-spill-batch pinned cb5d4612ef80 now 0a039a15846c
re-open candidates (0): none
dangling (0): none
$ meshwork cover sa-38wd6se --repin # and once it's fixed, a clean repin
sa-38wd6se covers docs/SPEC.md#sp-spill-batch @0a039a15846c
```

Nothing re-pins itself. It has to happen intentionally, hopefully after the task has been reimplemented to match the doc. The audit's other four rows are also traceability questions: spec clauses nobody covers, clauses only covered by dropped tasks, done tasks whose clause changed after they closed (surfaced as re-open candidates), and pins whose clause is gone. A clause ref can cross repos the same way a `docs:` link does.

## in a decentralized, federated mesh

Every repo keeps its own store, and one repo's queue doesn't care about another's...until it does. You can always express interdependencies as `project_name#task_id`.

With a lightweight [portfolio](docs/portfolios.md) (it's just a tiny git repo holding a `repos.toml`), meshwork can span locally cloned git repos. It can prioritize what work is ready across all of them, tracing blocking interdependencies. And you can query all your projects' tasks together with one SQL statement, views included. `portfolio stats` and `portfolio search` run the report and the text search over every store the same way. The the portfolio reads local checkouts only and never clones.

### that generates its own work

Dependencies points at work the other repo has already filed. meshwork projects can also ask for new work from other projects.

`--to <repo>` addresses a task to another project. nothing gets sent: the task stays in your store and surfaces in theirs through the porfolio querying all the local meshwork stores.

Let's say the spill fix needs the governor library to expose its wakeup interval:

```
$ meshwork add "Expose the wakeup interval as a config knob" --to governor --verify "contains config/engine.toml wakeup_ms"
sa-z1ecwc8
  docs/meshwork/sa-z1ecwc8-expose-the-wakeup-interval-as-a-config-knob.md
$ meshwork ready
sa-38wd6se  Fix spill batch sizing
sa-jt7zg9w  Write the spill postmortem
asks out (1):
sa-z1ecwc8  → governor  Expose the wakeup interval as a config knob  (0d)
```

Now it's out of your queue. It's the governor's problem now, and so it's listed under `asks out` with its age instead. The governor's clone finds it through the portfolio registry and shows it as addressed. It can take on the request by filing a task that answers it:

```
$ cd ../governor
$ meshwork ready
nothing ready
addressed to this repo (1):
demo#sa-z1ecwc8  Expose the wakeup interval as a config knob  (0d)
$ meshwork add "Expose the wakeup interval as a config knob" --cat api --answers demo#sa-z1ecwc8 --verify "contains src/config.rs wakeup_ms"
go-k9eeq4e
  docs/meshwork/go-k9eeq4e-expose-the-wakeup-interval-as-a-config-knob.md
$ meshwork ready
go-k9eeq4e  Expose the wakeup interval as a config knob
addressed to this repo (1):
demo#sa-z1ecwc8  Expose the wakeup interval as a config knob  (0d)  answered-by governor#go-k9eeq4e (open)
```

An open answer is an intent, not an answer. The request stays addressed until the task answering it is done, and comes back undone if that task gets dropped:

```
$ mkdir -p src && echo 'pub wakeup_ms: u64,' > src/config.rs   # stand-in for the work
$ meshwork close go-k9eeq4e
go-k9eeq4e open→done (verify exit 0)
$ cd ../demo
$ meshwork asks
asks in (0):
  (none)
asks out (1):
  sa-z1ecwc8  → governor  Expose the wakeup interval as a config knob  (0d)  answered-by governor#go-k9eeq4e (done)
```

Both sides watched that happen and nobody sent a message. `asks` is the whole inbox in both directions. `prime` and `ready` show the first few and name it for the rest. `prime`'s headline counts unanswered asks and the oldest one's age.

## boundaries

- **Zero network required.** A one-way, append-only GitHub mirror (issues created, comments appended, nothing ever edited or closed remotely) is planned at some point.
- **Never installs git hooks, never writes outside the repo.** The SessionStart hook that injects `prime` is Claude Code configuration you add yourself, once.
- **`verify:` is untrusted input.** Anything arriving by merge or hand-edit doesn't shell out until the checkout's operator approves the exact text (`close --approve`; `MESHWORK_TRUST=1` for checkouts reviewed before the runner touched them).
- **The CLI surface is frozen.** Anything not in the design doc's verb table is a non-goal, enforced by a test that diffs `--help` against the spec. Feature ideas default to the rejection list so this doesn't turn into Jira.
- **meshwork tracks meshwork.** This repo's own store holds its remaining roadmap, the repo's gate runs `lint` + `prime` against it on every push, and the digest you get when you open a session here is the one described above.

## getting it

Releases are darwin arm64, linux (arm64/x86_64), and windows x86_64.

Each consuming repo pins its own version:

```bash
echo "v0.5.0" > .meshwork-version     # commit this

VER=$(cat .meshwork-version)
DEST=~/.meshwork/versions/$VER
mkdir -p "$DEST"
gh release download "$VER" -R jbrjake/meshwork \
  -p "*aarch64-apple-darwin.tar.gz" -O - | tar -xz -C "$DEST"
"$DEST/meshwork" --help
```

Hooks and scripts invoke `~/.meshwork/versions/$(cat .meshwork-version)/meshwork`, so two repos can disagree. The adoption skill commits a two-line `./meshwork` shim so humans, hooks, and homunculi all reach the pinned version without re-deriving that path.

Building from source works too: `cargo install --git https://github.com/jbrjake/meshwork` (or `cargo build --release` in a clone).

### claude skill

The [plugin install](#install) up top is the skill — it teaches sessions the loop and handoff ritual above. From a local clone instead: `claude --plugin-dir /path/to/meshwork`. A repo that wants the skill's text pinned alongside its binary vendors the release tarball into its own `.claude/skills/`: [`.claude/skills/meshwork/references/install.md`](.claude/skills/meshwork/references/install.md).

---
Built on [DataFusion](https://datafusion.apache.org/).

MIT — see [LICENSE](LICENSE).
