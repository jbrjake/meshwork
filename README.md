# meshwork

**For when you make a real mesh out of things**

Opinionated minimalist task tracker for clankers. Tasks are human-readable markdown files tracked in git. There's a Rust CLI to manage them. No database: the CLI runs SQL queries directly against the markdown tasks. And it can materialize them into a byte-budgeted session-start digest to keep agents on track.

---

If you code with agents for more than toy projects, you manage your context window size rather than rely on compaction. You have to make sure new agents get on-boarded at session start and kept on-task.

You can get a lot done just with checkbox lists in markdown files like TODO.md, a HANDOFF.md to relay session context to fresh agents, and a project rule to 'pause at natural breaking points and persist relevant context to disk to handoff to a new session'. When it wraps up, you run `/clear`, then type 'Read the handoff and get up to speed, then continue building the project, committing and documenting as you go.'

Over time, though, you develop a lot of handoff context:

* Some tasks don't fully complete. Maybe you need to wait for a test box to become available for a benchmark, or you're blocked on a bug in another project, or you just switch focus to something shinier.
* Eventually you end up with multiple in-flight threads of work and the agent needs to be apprised of the state of all of them.
* Agents sometimes forget to mark items complete, or remove completed items.
* Agents are just...chatty. They will sometimes go into exhaustive detail on stuff no one cares about.

It becomes a problem over time because of the context hit. These files get read in all the time. Extraneous stuff costs tokens and focus. So you add a rule to the project to rotate old items to an archive.

Of course, agents aren't reliable at following rules. So you add a hook with a deterministic trigger: you cannot commit code if your TODO.md or HANDOFF.md are above N lines. Of course they'll game that and have super long lines, so you change to characters. Either way, it's a game to them and they play golf trying to get under-but-as-close-to the limit as possible. Except...they're terrible at counting. So it can take them like 3-4 tries to cut the text down to the limit you set. Eventually it gets so time-consuming and token-expensive that you raise the ceiling when important stuff has to get tracked.

None of this is hypothetical.

*Measured 2026-08-04, across the all-time history of two working repos.*

| | sazed | leras |
|---|---|---|
| Edit calls targeting TODO.md + HANDOFF.md, share of **all edits ever made** | 19.75% (927 of 4,693) | ~16% (641) |
| sessions reading both files within their first 5 tool calls | ~95%, at a ~22K-token tax each | ~90% |
| worst line-cap thrash episode | 34% of a session's shell calls; 14 gate failures in 84 minutes | 26 consecutive tool calls |
| cap raises (every one reactive, none structural) | 500→550 | 200→300→450→600 **in 8 days** |

One repo's CLAUDE.md proudly declared its worklist was "131 lines." It was 38KB.

Measured again after sazed — the first repo above — migrated onto meshwork (2026-08-07, v0.1.5, store of 123 tasks):

| | before | after |
|---|---|---|
| session-start onboarding read | TODO.md + HANDOFF.md: **116,119 bytes** (~28K tokens), read in full as the first two tool calls of ~every session | `meshwork prime`: **2,968 bytes** (~700 tokens), injected by the SessionStart hook before the first tool call — **39× less** |
| tracker busywork per session — every tool call spent reading, editing, or policing the tracker instead of working | **~33.3K tokens**, 28.8% of the session's tool traffic: 1 busywork token per 2.5 of work (average of the final 10 pre-migration sessions) | **~9.8K tokens**, 8.6%: 1 per 10.6 — **3.4× less**, and the freed traffic went to work (82K → 104K work tokens) |
| getting oriented, observed in the first post-migration session | 2 whole-file reads before any work | four short store reads (`show`, `why`, `git log`), zero doc sweeps, `start` on the recommended task 8 minutes in |
| handoff corpus | **84 HANDOFF files, 841KB**, accumulated over 5 weeks, plus 27 session-end commits rewriting two files | HANDOFF.md deleted; `handoff:` lives on the task it describes, `prime` materializes the one that's next |
| worklist fidelity | one 550-line TODO.md, with its own carve-out in the repo's line-cap gate | 123 tasks, 23 dependency edges, 108 runnable `verify:` gates, `lint` 0 errors 0 warnings |

*Busywork counted from the session transcripts by `scripts/admin-tokens.py`: every tool call (input + result) whose command or path touches the tracker — TODO.md/HANDOFF reads, edits, and cap-check thrash before; every meshwork CLI call, store file edit, and console todo mirroring after — at 4 chars/token. The classification errs against meshwork: its whole CLI surface counts as busywork.*

> Every terminal transcript below is pasted from a real run of the binary.

---

## Is this for you?

| reach for meshwork when… | reach for something else when… |
| --- | --- |
| the work happens in **many short agent sessions** that must hand off cleanly | one long-lived human holds the context in their head |
| tasks have a **runnable definition of done** (`verify:` a test, a grep, a file) | done-ness is a conversation |
| **git is the source of truth** and work must survive offline, in worktrees, across machines | you need webhooks, boards, assignees, notifications |
| one owner, a portfolio of repos | a team that needs permissions and a web UI |
| you want the tracker's own output **capped in bytes**, because context is the scarce resource | context is free because a human is reading |

---

## The session loop

Adopt in an existing repo (`meshwork init`).
Or migrate an existing TODO.md (`meshwork import todo TODO.md`), and each checkbox becomes a task file.

File work as tasks with real dependencies and a runnable definition of done.

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

(Filing a whole interlinked batch at once is `add --batch`: a stream of task documents on stdin, `@handle` refs between siblings that don't have ids yet, all files written or none.)

Then you can ask it what's actionable:

```
$ meshwork ready
sa-nmvpyqr  Reproduce the 600M-row spill cliff
sa-jt7zg9w  Write the spill postmortem
```

The blocked task doesn't appear, and you can ask why:

```
$ meshwork why sa-38wd6se
sa-38wd6se blocked by 1:
- sa-nmvpyqr (open) — verify: test -f repro.log
```

Try to close something that isn't actually done, and the first refusal isn't even about the work:

```
$ meshwork close sa-jt7zg9w
meshwork: refusing unapproved verify for sa-jt7zg9w (MW-E5, DESIGN §12b)
  verify: test -f docs/postmortem.md
  task files arrive via merge and are untrusted; review the command, then:
  meshwork close sa-jt7zg9w --approve   (records approval for this clone)
  reviewed checkouts (CI, gates) may grant MESHWORK_TRUST=1 instead
```

Task files are data that arrive through git — a merge, a PR, a synced clone — and `verify:` is the only field the tool ever executes. Whoever can land a file in your repo controls every line of it. So shell verifies are trust-on-first-use, like direnv: you approve the exact text, per task, and the approval is recorded per clone, outside git, where a merge can't plant one. Approve it, and the tool gets to the real objection — a task's `verify:` command must exit 0, observed, right now:

```
$ meshwork close sa-jt7zg9w --approve
approving verify for sa-jt7zg9w (this clone only, MW-E5):
  verify: test -f docs/postmortem.md
meshwork: sa-jt7zg9w stays open: verify exit 1 (`test -f docs/postmortem.md`)
```

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

Session's over. Before wrapping up, leave a note on whichever task is up next — the one authored piece of the handoff:

```
$ meshwork set sa-38wd6se --handoff "Cliff is governor wakeup, not batch size — don't burn a session re-deriving that (comment on sa-nmvpyqr has the repro). Try wakeup=250ms before touching batch math."
sa-38wd6se handoff set
```

The benefit of working this way isn't any magical belief that your vibe-coded slop works. That's what testing is for. It's so the next session doesn't have to read files to catch up or waste time at the end of sessions rotating tasks in text files. It gets `prime` — injected automatically by a SessionStart hook — which materializes the handoff from the store:

```
$ meshwork prime
sazed — 2 open, 1 done
store @ a4912f1
engine/spill 1 · docs 1
next → sa-38wd6se Fix spill batch sizing
  » Cliff is governor wakeup, not batch size — don't burn a session
  » re-deriving that (comment on sa-nmvpyqr has the repro). Try wakeup=250ms
  » before touching batch math.
  [engine/spill]
  verify: cargo test spill::batch
also ready (1 more, top 1):
- sa-jt7zg9w Write the spill postmortem
recently done:
- 2026-08-06T21:58Z sa-nmvpyqr Reproduce the 600M-row spill cliff
```

Almost everything in that digest is derived from the task files — counts, the category rollup, what's next and why, what just finished. The `store @` line is derived too, from git: a session landing on a stale clone sees uncommitted task edits and drift from upstream up front instead of discovering them mid-work. The exception is the `»` lines. That's the `handoff:` block, the outgoing session's voice to the incoming one, and it's the only authored part of the handoff. It lives on whichever task is up next. Linting warns if you leave one on a task you close.

The digest is capped at 6KB ≈ 1.5K tokens versus the 22K-token ritual it replaces. When a store is big enough to threaten the cap, the truncation is explicit.

---

## Files are the API

A task is one markdown file. This is the entire storage format — `sa-38wd6se`, the blocked task from the session loop, verbatim:

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

Hand-editing is legal and expected but never necessary. Fields can all be set by the CLI — flags on `add` at creation, `meshwork set <id>` after. `meshwork lint` validates the result (schema, cycles, dangling edges, post-merge damage; `lint --fix` repairs the mechanical cases). A file that fails to parse isn't dropped. It shows up as an `invalid` row in every listing until someone fixes it.

The format has a normative spec, [FORMAT.md](FORMAT.md) — versioned, self-contained, the thing a third-party reader implements from without ever running the binary. Where the spec and the binary disagree, the spec wins.

When a task reaches `done` or `dropped`, its file moves to `docs/meshwork/archive/` automatically (and moves back on `reopen`). Archived tasks stay loaded and queryable — dependency resolution, SQL, and the digest are location-blind; only the clutter leaves the store root.

Because tasks are files in git, concurrency is git's problem. Two sessions in separate worktrees can create tasks, comment on the *same* task, and close tasks, then merge without manual conflict resolution. The one merge artifact git can produce (a duplicated frontmatter key from union-merge) is repaired by `lint --fix`. Minted stamps carry UTC minute resolution, so interleaved same-day appends keep a recoverable order. And `start` records who claimed the task — `claimed-by:`, the same self-professed identity comments use. It's advisory: a coordination signal between parallel sessions, never a lock, and a post-merge double-claim is a lint finding like the rest.

The `seq` field is the prioritization: integers with gaps of 10, lower runs sooner.

For anything the canned verbs don't answer, the store is SQL-addressable:

```
$ meshwork q "SELECT category, count(*) AS n FROM tasks WHERE status='open' GROUP BY category ORDER BY n DESC"
category | n
docs | 1
engine/spill | 1
(2 rows)
```

The `## log` lines are a table too — every status transition, stamped — so blocked-duration and cycle-time queries are one `SELECT` away.

Every verb also takes `--json` for scripts and agents; the envelope stamps its schema version in-band, because per-repo pinning makes mixed binary versions the normal case.

---

## Boundaries

- **Zero network required.** A one-way, append-only GitHub mirror — issues created, comments appended, nothing ever edited or closed remotely — is designed and queued.
- **Never installs git hooks, never writes outside the repo.** The SessionStart hook that injects `prime` is Claude Code configuration you add yourself, once.
- **`verify:` is untrusted input.** Nothing shells out until this clone's operator approves the exact text (`close --approve`; `MESHWORK_TRUST=1` for checkouts reviewed before the runner touched them). Git authorship is never a trust signal — it's a self-professed string.
- **The CLI surface is frozen.** Anything not in the design doc's verb table is a non-goal, enforced by a test that diffs `--help` against the spec. Feature ideas default to the rejection list so this doesn't turn into Jira.
- **meshwork tracks meshwork.** This repo's own store holds its remaining roadmap, the repo's gate runs `lint` + `prime` against it on every push, and the digest you get when you open a session here is the one described above.

---

## Getting it

Each consuming repo pins its own version:

```bash
echo "v0.1.5" > .meshwork-version     # commit this

VER=$(cat .meshwork-version)
DEST=~/.meshwork/versions/$VER
mkdir -p "$DEST"
gh release download "$VER" -R jbrjake/meshwork \
  -p "*aarch64-apple-darwin.tar.gz" -O - | tar -xz -C "$DEST"
"$DEST/meshwork" --help
```

Hooks and scripts invoke `~/.meshwork/versions/$(cat .meshwork-version)/meshwork`, so two repos can disagree.

If you drive repos with Claude Code, install the bundled skill so sessions follow the loop and handoff ritual above: [`.claude/skills/meshwork/references/install.md`](.claude/skills/meshwork/references/install.md). Releases are darwin arm64, linux (arm64/x86_64), and windows x86_64.

---
Built on [DataFusion](https://datafusion.apache.org/).

MIT — see [LICENSE](LICENSE).
