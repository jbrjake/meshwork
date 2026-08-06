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

> Every terminal transcript below is pasted from a real run of the released binary.

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
sa-k733xy9
  docs/meshwork/sa-k733xy9-reproduce-the-600m-row-spill-cliff.md

$ cat docs/meshwork/sa-k733xy9-reproduce-the-600m-row-spill-cliff.md
---
id: sa-k733xy9
title: Reproduce the 600M-row spill cliff
status: open
category: engine/spill
verify: test -f repro.log
created: 2026-08-06
---

## log
- 2026-08-06 created
```

We can also add tasks with dependencies and an explicit `seq` (the priority weight). And note how categories are hierarchical, with slashes separating the levels:

```
$ meshwork add "Fix spill batch sizing" --cat engine/spill --needs sa-k733xy9 --seq 10 --verify "cargo test spill::batch"
sa-q07thzc
  docs/meshwork/sa-q07thzc-fix-spill-batch-sizing.md
$ meshwork add "Write the spill postmortem" --cat docs --verify "test -f docs/postmortem.md"
sa-dv8y8wa
  docs/meshwork/sa-dv8y8wa-write-the-spill-postmortem.md
```

Then you can ask it what's actionable:

```
$ meshwork ready
sa-dv8y8wa  Write the spill postmortem
sa-k733xy9  Reproduce the 600M-row spill cliff
```

The blocked task doesn't appear, and you can ask why:

```
$ meshwork why sa-q07thzc
sa-q07thzc blocked by 1:
- sa-k733xy9 (open) — verify: test -f repro.log
```

Try to close something that isn't actually done, and the tool declines. A task's `verify:` command must exit 0, observed, right now.

```
$ meshwork close sa-dv8y8wa
meshwork: sa-dv8y8wa stays open: verify exit 1 (`test -f docs/postmortem.md`)
```

So let's get to work:

```
$ meshwork start sa-k733xy9
sa-k733xy9 open→doing
$ meshwork comment sa-k733xy9 --as claude "cliff reproduces at batch=64k; tracks the governor wakeup interval, not batch size"
sa-k733xy9: comment added as [claude]
$ touch repro.log        # stand-in for the actual work
$ meshwork close sa-k733xy9
sa-k733xy9 doing→done (verify exit 0)
```

As you can see, there are limits to enforcement. An agent absolutely will reach for that stand-in and just touch the file to hit the requirement. meshwork isn't guaranteeing anything more than that the provided validation passes.

(`close --waive "reason"` exists for the genuinely unverifiable. It's recorded and queryable as `WHERE waived IS NOT NULL` so it's loud and visible.)

Session's over. Before wrapping up, leave a note on whichever task is up next — the one authored piece of the handoff:

```
$ meshwork set sa-q07thzc --handoff "Cliff is governor wakeup, not batch size — don't burn a session re-deriving that (comment on sa-k733xy9 has the repro). Try wakeup=250ms before touching batch math."
sa-q07thzc handoff set
```

The benefit of working this way isn't any magical belief that your vibe-coded slop works. That's what testing is for. It's so the next session doesn't have to read files to catch up or waste time at the end of sessions rotating tasks in text files. It gets `prime` — injected automatically by a SessionStart hook — which materializes the handoff from the store:

```
$ meshwork prime
sazed — 2 open, 1 done
engine/spill 1 · docs 1
next → sa-q07thzc Fix spill batch sizing
  » Cliff is governor wakeup, not batch size — don't burn a session
  » re-deriving that (comment on sa-k733xy9 has the repro). Try wakeup=250ms
  » before touching batch math.
  [engine/spill]
  verify: cargo test spill::batch
also ready (1 more, top 1):
- sa-dv8y8wa Write the spill postmortem
recently done:
- 2026-08-06 sa-k733xy9 Reproduce the 600M-row spill cliff
```

Almost everything in that digest is derived from the task files — counts, the category rollup, what's next and why, what just finished. The exception is the `»` lines. That's the `handoff:` block, the outgoing session's voice to the incoming one, and it's the only authored part of the handoff. It lives on whichever task is up next. Linting warns if you leave one on a task you close.

The digest is capped at 6KB ≈ 1.5K tokens versus the 22K-token ritual it replaces. When a store is big enough to threaten the cap, the truncation is explicit.

---

## Files are the API

A task is one markdown file. This is the entire storage format — `sa-q07thzc`, the blocked task from the session loop, verbatim:

```markdown
---
id: sa-q07thzc
title: Fix spill batch sizing
status: open
category: engine/spill
needs: [sa-k733xy9]
verify: cargo test spill::batch
seq: 10
created: 2026-08-06
handoff: |
  Cliff is governor wakeup, not batch size — don't burn a session
  re-deriving that (comment on sa-k733xy9 has the repro). Try wakeup=250ms
  before touching batch math.
---

## log
- 2026-08-06 created
```

Hand-editing is legal and expected but never necessary. Fields can all be set by the CLI — flags on `add` at creation, `meshwork set <id>` after. `meshwork lint` validates the result (schema, cycles, dangling edges, post-merge damage; `lint --fix` repairs the mechanical cases). A file that fails to parse isn't dropped. It shows up as an `invalid` row in every listing until someone fixes it.

When a task reaches `done` or `dropped`, its file moves to `docs/meshwork/archive/` automatically (and moves back on `reopen`). Archived tasks stay loaded and queryable — dependency resolution, SQL, and the digest are location-blind; only the clutter leaves the store root.

Because tasks are files in git, concurrency is git's problem. Two sessions in separate worktrees can create tasks, comment on the *same* task, and close tasks, then merge without manual conflict resolution. The one merge artifact git can produce (a duplicated frontmatter key from union-merge) is repaired by `lint --fix`.

The `seq` field is the prioritization: integers with gaps of 10, lower runs sooner.

For anything the canned verbs don't answer, the store is SQL-addressable:

```
$ meshwork q "SELECT category, count(*) AS n FROM tasks WHERE status='open' GROUP BY category ORDER BY n DESC"
category | n
docs | 1
engine/spill | 1
(2 rows)
```

Every verb also takes `--json` with a stable schema, for scripts and agents.

---

## Boundaries

- **Zero network required.** A one-way, append-only GitHub mirror — issues created, comments appended, nothing ever edited or closed remotely — is designed and queued.
- **Never installs git hooks, never writes outside the repo.** The SessionStart hook that injects `prime` is Claude Code configuration you add yourself, once.
- **The CLI surface is frozen.** Anything not in the design doc's verb table is a non-goal, enforced by a test that diffs `--help` against the spec. Feature ideas default to the rejection list so this doesn't turn into Jira.
- **meshwork tracks meshwork.** This repo's own store holds its remaining roadmap, the repo's gate runs `lint` + `prime` against it on every push, and the digest you get when you open a session here is the one described above.

---

## Getting it

Each consuming repo pins its own version:

```bash
echo "v0.1.4" > .meshwork-version     # commit this

VER=$(cat .meshwork-version)
DEST=~/.meshwork/versions/$VER
mkdir -p "$DEST"
gh release download "$VER" -R jbrjake/meshwork \
  -p "*aarch64-apple-darwin.tar.gz" -O - | tar -xz -C "$DEST"
"$DEST/meshwork" --help
```

Hooks and scripts invoke `~/.meshwork/versions/$(cat .meshwork-version)/meshwork`, so two repos can disagree.

If you drive repos with Claude Code, install the bundled skill so sessions follow the loop and handoff ritual above: [`.claude/skills/meshwork/references/install.md`](.claude/skills/meshwork/references/install.md). Releases are darwin arm64 today. Linux and Windows builds are queued in the store.

---
Built on [DataFusion](https://datafusion.apache.org/).

MIT — see [LICENSE](LICENSE).
