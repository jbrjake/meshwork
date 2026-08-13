# meshwork

**for when you make a real mesh out of things**

Opinionated minimalist todo list for clankers. Tasks get their own human-readable markdown files logged in git. There's a Rust CLI to manage them. No database: the CLI runs SQL queries directly against the markdown. And it can materialize them into a byte-budgeted session-start digest to keep agents on track.

It's a mesh twice over: because tasks are modeled as a graph with edges to related tasks, and also because the git repos federate. A task in one project can depend on a task in another, and you can set up a portfolio view spanning multiple repos.

## install

```
/plugin marketplace add jbrjake/claude-plugin-marketplace
/plugin install meshwork@jbrjake
```

That installs the skill into Claude Code. Ask a session to "adopt meshwork in this repo" and it handles the rest ([getting it](#getting-it) has the manual path).

## quick-start

```bash
$ meshwork init

$ meshwork add "Do the thing with the stuff" --cat stuff/doodads --verify "cargo test stuff::thing"
ac-acnxdkg
  docs/meshwork/ac-acnxdkg-do-the-thing-with-the-stuff.md

$ meshwork prime
acme — 1 open
store @ c601195 · 1 uncommitted task edit
stuff/doodads 1
next → ac-acnxdkg Do the thing with the stuff
  [stuff/doodads]
  verify: cargo test stuff::thing

$ meshwork start ac-acnxdkg --as claude
ac-acnxdkg open→doing
$ meshwork comment ac-acnxdkg --as claude "Smoking gun! You're absolutely right. This seam is load-bearing. On it. Cerebrating..."
ac-acnxdkg: comment added as [claude]
...
$ meshwork close ac-acnxdkg --approve
approving verify for ac-acnxdkg (this clone only, MW-E5):
  verify: cargo test stuff::thing

running 1 test
test stuff::thing ... ok

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
verify: cargo test stuff::thing
created: 2026-08-12T21:28Z
---

## log
- 2026-08-12T21:28Z created
- 2026-08-12T21:29Z open→doing — claimed by claude
- 2026-08-12T21:29Z doing→done — verify exit 0 @ c601195+5

## comments
- 2026-08-12T21:29Z [claude] Smoking gun! You're absolutely right. This seam is load-bearing. On it. Cerebrating...
```

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

*Measured 2026-08-04, across the all-time history of two working repos.*

| | Project A | Project B |
|---|---|---|
| Edit calls targeting TODO.md + HANDOFF.md | ~20% (927 of 4,693) | ~16% (641) |
| sessions reading both files within their first 5 tool calls | ~95%, at a ~22K-token tax each | ~90% |
| worst line-cap thrash episode | 34% of a session's shell calls (14 gate failures in 84 minutes) | 26 consecutive tool calls |
| line limit increases | 500→550 | 200→750 |

One repo's CLAUDE.md proudly declared its worklist was "131 lines." It was 38KB.

### with meshwork

*Measured after both repos migrated to meshwork (Project A 2026-08-07, Project B 2026-08-10). Each table: that repo's last 10 sessions before meshwork vs its working sessions after migrating.*

*Every turn repeats the entire conversation, so the first things in context get repeated the most. Project A's pre-migration sessions opened by reading ~28K tokens of TODO.md + HANDOFF.md. Re-paid on all 153 requests after it, that one read compounds to ~4M tokens across all 153 turns after that. And that's before you account for all the TODO and HANDOFF edits the agent makes during the session. Altogether it's 4.19M tokens. The same content landing at request 150 would have compounded to ~0.1M tokens.

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

*The repos are developed with different models (A: Opus, B: Fable), so compare before/after within a table, not across the two.*

Four million accrued busywork tokens across turns is a drop in the bucket for any serious coding session. This isn't about cost savings. It's about the time lost to all those turns while the agent thrashes against a todo list in markdown, and the lost focus of bringing extraneous content into the context window. Interestingly, after migration to meshwork the freed context went to work: Project A sessions carry 90K → 113K real work tokens/session and run ~28% longer (154 → 197 useful requests/session).

## when meshwork makes sense

| reach for meshwork when… | reach for something else when… |
| --- | --- |
| the work happens in **many short agent sessions** that must hand off cleanly | one long-lived human holds the context in their head |
| tasks have a **runnable definition of done** (`verify:` a test, a grep, a file) | done-ness is a conversation |
| **git is the source of truth** and work must survive offline, in worktrees, across machines | you need webhooks, boards, assignees, notifications |
| one owner, a portfolio of repos | a team that needs permissions and a web UI |
| you want the tracker's own output **capped in bytes**, because context is the scarce resource | context is free because a human is reading |

## the workflow

> Every terminal transcript below is pasted from a real run of the binary.

### bootstrapping

Adopt in an existing repo with `meshwork init`.
Or migrate an existing TODO.md with `meshwork import todo TODO.md`, and each checkbox becomes a task file. Nested checkboxes get parent/child relationships.

### adding tasks

File work as tasks with real dependencies and a runnable definition of done:

```
$ meshwork add "Reproduce the 600M-row spill cliff" --cat engine/spill --verify "test -f repro.log"
sa-nmvpyqr
  docs/meshwork/sa-nmvpyqr-reproduce-the-600m-row-spill-cliff.md

$ cat docs/meshwork/sa-nmvpyqr-reproduce-the-600m-row-spill-cliff.md
---
id: sa-nmvpyqr
title: Reproduce the 600M-row spill cliff
status: open
category: engine/spill
verify: test -f repro.log
created: 2026-08-06T21:47Z
---

## log
- 2026-08-06T21:47Z created
```

We can also add tasks with dependencies and an explicit `seq` (the priority weight). And note how categories are hierarchical, with slashes separating the levels:

```
$ meshwork add "Fix spill batch sizing" --cat engine/spill --needs sa-nmvpyqr --seq 10 --verify "cargo test spill::batch"
sa-38wd6se
  docs/meshwork/sa-38wd6se-fix-spill-batch-sizing.md
$ meshwork add "Write the spill postmortem" --cat docs --verify "test -f docs/postmortem.md"
sa-jt7zg9w
  docs/meshwork/sa-jt7zg9w-write-the-spill-postmortem.md
```

You can draft a task without `--verify`, but one's got to exist to start it, and the command has to succeed to complete it.

#### batching

Filing a whole interlinked batch at once is `add --batch`: a stream of task documents on stdin, `@handle` refs between siblings that don't have ids yet, all files written or none. Each document is the same frontmatter a task file carries, minus the `id:` — meshwork mints those:

```
$ meshwork add --batch - <<'EOF'
---
handle: bench
title: Benchmark spill at 64k-1M batch sizes
category: engine/spill
needs: [sa-38wd6se]
verify: test -f bench/spill.csv
---
Numbers before and after the sizing fix.
---
title: Tune the governor wakeup default
category: engine/governor
needs: [@bench]
verify: cargo test governor::wakeup_default
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
- sa-nmvpyqr (open) — verify: test -f repro.log
```

### closing tasks

Try to close something, and the first refusal isn't even about the work:

```
$ meshwork close sa-jt7zg9w
meshwork: refusing unapproved verify for sa-jt7zg9w (MW-E5, DESIGN §12b)
  verify: test -f docs/postmortem.md
  task files arrive via merge and are untrusted; review the command, then:
  meshwork close sa-jt7zg9w --approve   (records approval for this clone)
  reviewed checkouts (CI, gates) may grant MESHWORK_TRUST=1 instead
```

Task files can come from untrusted sources, like third-party PRs. `verify:` fields are executed in the shell. This is not a fantastic combination for security. So shell verifies are trust-on-first-use: you approve the exact text, per task, and the approval is recorded per clone, outside git, where a merge can't plant one.

What this means is the human in the loop is responsible for security. If you accept tasks from other people, make sure you read the contents of anything they will execute before you approve. If you just hit your enter key to every Claude prompt, all bets are off.

Anyway, approve the verification run, and meshwork gets to the real objection. A task's `verify:` command must be witnessed exiting with code 0, and this one wasn't, since the postmortem hasn't actually been written.

```
$ meshwork close sa-jt7zg9w --approve
approving verify for sa-jt7zg9w (this clone only, MW-E5):
  verify: test -f docs/postmortem.md
meshwork: sa-jt7zg9w stays open: verify exit 1 (`test -f docs/postmortem.md`)
```

### work loop

So let's get to work:

```
$ meshwork start sa-nmvpyqr --as claude
sa-nmvpyqr open→doing
$ meshwork comment sa-nmvpyqr --as claude "cliff reproduces at batch=64k; tracks the governor wakeup interval, not batch size"
sa-nmvpyqr: comment added as [claude]
$ touch repro.log        # stand-in for the actual work
$ meshwork close sa-nmvpyqr --approve
approving verify for sa-nmvpyqr (this clone only, MW-E5):
  verify: test -f repro.log
sa-nmvpyqr doing→done (verify exit 0)
```

As you can see, there are limits to enforcement. An agent absolutely will reach for that stand-in and just touch the file to hit the requirement. meshwork isn't guaranteeing anything more than that the provided validation passes.

(`close --waive "reason"` exists for the genuinely unverifiable. It's recorded and queryable as `WHERE waived IS NOT NULL` so it's loud and visible.)

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
next → sa-38wd6se Fix spill batch sizing
  » Cliff is governor wakeup, not batch size — don't burn a session
  » re-deriving that (comment on sa-nmvpyqr has the repro). Try wakeup=250ms
  » before touching batch math.
  [engine/spill] · blocks: sa-bbds8pt
  verify: cargo test spill::batch
also ready (1 more, top 1):
- sa-jt7zg9w Write the spill postmortem
recently done:
- 2026-08-06T21:58Z sa-nmvpyqr Reproduce the 600M-row spill cliff
```

Almost everything in that digest is derived from the task files: counts, the category rollup, what's next and why, what just finished. The `store @` line is derived too, from git. A session landing on a stale clone sees uncommitted task edits and drift from upstream up front instead of discovering them mid-work. The exception is the `»` lines. That's the `handoff:` block. It lives on whichever task is up next. Linting warns if you leave one on a task you close.

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
verify: cargo test spill::batch
seq: 10
created: 2026-08-06T21:48Z
handoff: |
  Cliff is governor wakeup, not batch size — don't burn a session
  re-deriving that (comment on sa-nmvpyqr has the repro). Try wakeup=250ms
  before touching batch math.
---

## log
- 2026-08-06T21:48Z created
```

Hand-editing is legal and expected but never necessary. Fields can all be set by the CLI with `meshwork add` at creation and `meshwork set <id>` after. `meshwork lint` validates them (schema, cycles, dangling edges, post-merge damage), and `meshwork lint --fix` repairs what it can. A file that fails to parse isn't dropped. It shows up as an `invalid` row in every listing until someone fixes it.

The format has a spec, [FORMAT.md](FORMAT.md), that's versioned and self-contained. Anyone can implement against it without having to embrace this project's code.

When a task reaches `done` or `dropped`, to de-clutter, its file moves to `docs/meshwork/archive/` automatically (and moves back on `reopen`). Archived tasks stay loaded and queryable. Dependency resolution, SQL, and the digest are location-blind.

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

The `## log` lines are a table too, with every status transition timestamped, so that includes questions like 'how long have tasks been blocked?' and 'how long do tasks take to complete?'

The CLI also has a `--json` flag for scripts and agents.

## in a decentralized, federated mesh

Every repo keeps its own store, and one repo's queue doesn't care about another's...until it does.

### portfolio setup
You can set up a portfolio to look at them together, with a tiny git repo holding a `repos.toml`:

```toml
[[repo]]
name = "alpha"
remote = "git@github.com:example/alpha.git"

[[repo]]
name = "beta"
remote = "git@github.com:example/beta.git"

[[repo]]
name = "gamma"
remote = "git@github.com:example/gamma.git"
```

### portfolio usage

`portfolio ready` shows tasks from all repos in the .toml that you've got locally:

```
$ meshwork portfolio ready | head -4
portfolio: skipped gamma — no checkout at /Users/dev/Documents/code/gamma
beta#bz-s3q1  Schema qualifier cleanup
alpha#az-n33d  Publish spill report
alpha#az-x9b2  Cross-repo consumer bump
alpha#az-r3l8  Document spill knobs
```

### inter-dependencies

Dependencies cross repos: the beta repo shipped its reader rewrite (`bz-c0r3`, done), and the alpha repo's consumer bump depends on it:

```
$ grep needs: docs/meshwork/az-x9b2-cross-repo-consumer-bump.md
needs: [beta#bz-c0r3]
$ meshwork why az-x9b2
az-x9b2: nothing blocking — every hard dep is done/dropped
```

That works from inside individual repos, no portfolio command involved. If it can't find the other repo on disk, it'll let you know:

```
$ meshwork why az-x9b2
az-x9b2 blocked by 1:
- beta#bz-c0r3 (unresolved — absent or unregistered repo)
```

Only a done/dropped task on the other side satisfies the dependency.

### cross-prioritization

The portfolio repo can also hold a `sequence.md`, a list of `repo#id` bullets under cosmetic section headings:

```markdown
## Tranche 1 — spill cliff before anything

- alpha#az-t5k1
- beta#bz-r34d

## Tranche 2 — reporting

- alpha#az-n33d
```

`portfolio next` answers the session-start question across everything: what single task is next? The first *ready* sequenced task wins; `az-t5k1` is already claimed as `doing`, so:

```
$ meshwork portfolio next
portfolio: skipped gamma — no checkout at /Users/dev/Documents/code/gamma
beta#bz-r34d  Retry policy for fetch
```

Ready tasks missing from the sequence fall back to `repos.toml` order, then per-repo `seq`. Resequencing an entire portfolio is editing one small file in one small repo, reviewed and diffed like everything else.

### unified querying

`portfolio q` is the same SQL surface with a `repo` column.

### portfolio pathing

Per-machine checkout paths live in a gitignored `repos.local.toml` (default: `~/Documents/code/<name>`; the portfolio dir itself defaults to `~/Documents/code/portfolio`, `MESHWORK_PORTFOLIO` overrides).

### portfolio performance

Cold, `ready` over a 1K-task store, and the union across 20 repos, both answer in ~30ms.

## boundaries

- **Zero network required.** A one-way, append-only GitHub mirror (issues created, comments appended, nothing ever edited or closed remotely) is designed and queued.
- **Never installs git hooks, never writes outside the repo.** The SessionStart hook that injects `prime` is Claude Code configuration you add yourself, once.
- **`verify:` is untrusted input.** Nothing shells out until this clone's operator approves the exact text (`close --approve`; `MESHWORK_TRUST=1` for checkouts reviewed before the runner touched them).
- **The CLI surface is frozen.** Anything not in the design doc's verb table is a non-goal, enforced by a test that diffs `--help` against the spec. Feature ideas default to the rejection list so this doesn't turn into Jira.
- **meshwork tracks meshwork.** This repo's own store holds its remaining roadmap, the repo's gate runs `lint` + `prime` against it on every push, and the digest you get when you open a session here is the one described above.

## getting it

Releases are darwin arm64, linux (arm64/x86_64), and windows x86_64.

Each consuming repo pins its own version:

```bash
echo "v0.2.0" > .meshwork-version     # commit this

VER=$(cat .meshwork-version)
DEST=~/.meshwork/versions/$VER
mkdir -p "$DEST"
gh release download "$VER" -R jbrjake/meshwork \
  -p "*aarch64-apple-darwin.tar.gz" -O - | tar -xz -C "$DEST"
"$DEST/meshwork" --help
```

Hooks and scripts invoke `~/.meshwork/versions/$(cat .meshwork-version)/meshwork`, so two repos can disagree. The adoption skill commits a two-line `./meshwork` shim so humans, hooks, and homunculi all reach the pinned version without re-deriving that path.

### claude skill

The [plugin install](#install) up top is the skill — it teaches sessions the loop and handoff ritual above. From a local clone instead: `claude --plugin-dir /path/to/meshwork`. A repo that wants the skill's text pinned alongside its binary vendors the release tarball into its own `.claude/skills/`: [`.claude/skills/meshwork/references/install.md`](.claude/skills/meshwork/references/install.md).

---
Built on [DataFusion](https://datafusion.apache.org/).

MIT — see [LICENSE](LICENSE).
