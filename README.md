# meshwork

**Your task graph was always a graph. Stop storing it as prose.**

Task tracking for repos whose contributors are AI agents and one human. Tasks are markdown files in git, queried with SQL, materialized into a byte-budgeted session-start digest. One Rust binary. No database, no daemon, no web app, no network.

---

Here's how it goes without this. Your repo has a `TODO.md` and a `HANDOFF.md`, and every agent session starts the same way: read both files, all of them, to find out what's going on. Then work happens. Then the session ends by *rewriting* both files so the next session can repeat the ritual. The files grow, so you add a line cap. The cap gets hit, so sessions compress the prose instead of doing work. Then somebody raises the cap.

None of this is hypothetical. It was measured, in this portfolio, before meshwork existed:

*Measured 2026-08-04, across the all-time history of two working repos.*

| | sazed | leras |
|---|---|---|
| Edit calls targeting TODO.md + HANDOFF.md, share of **all edits ever made** | 19.75% (927 of 4,693) | ~16% (641) |
| sessions reading both files within their first 5 tool calls | ~95%, at a ~22K-token tax each | ~90% |
| worst line-cap thrash episode | 34% of a session's shell calls; 14 gate failures in 84 minutes | 26 consecutive tool calls |
| cap raises (every one reactive, none structural) | 500→550 | 200→300→450→600 **in 8 days** |

One repo's CLAUDE.md proudly declared its worklist was "131 lines." It was 38KB. Line caps don't measure anything an LLM pays for — tokens are bytes — so the caps got gamed, in good faith, by every session that hit one.

And the thing is, the dependency structure was *in* the prose the whole time — "after the seam extraction lands", "blocked on the parquet fix upstream" — written down and unqueryable. A fifth of all editing effort went into maintaining a graph as paragraphs.

Each measured failure dictates a piece of the fix:

- Edges live in prose → **edges become data** (`needs:`), and "what can I work on" becomes a query instead of a reading-comprehension exercise.
- Every session pays a 22K-token entry tax → **the digest is materialized and byte-budgeted**: `prime` renders ≤6KB from the store, injected at session start.
- Line caps get gamed → **budgets are in bytes**, enforced by the tool, on the tool's own output and on task bodies.
- "Done" was whatever the prose said → **closing a task runs its `verify:` command** and refuses on nonzero exit.

That's the whole tool. It's the failure list, inverted.

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

Adopt in an existing repo (`meshwork init`), or migrate an existing TODO.md (`meshwork import todo TODO.md` — checkboxes become task files, one shot, review the diff). Then the loop looks like this.

File work as tasks with real dependencies and a runnable definition of done:

```
$ meshwork add "Reproduce the 600M-row spill cliff" --cat engine/spill --verify "test -f repro.log"
sa-pdk6
$ meshwork add "Fix spill batch sizing" --cat engine/spill --needs sa-pdk6 --verify "cargo test spill::batch"
sa-vj0d
$ meshwork add "Write the spill postmortem" --cat docs --verify "test -f docs/postmortem.md"
sa-6r27
```

Ask what's actionable. The blocked task doesn't appear, and you can ask why:

```
$ meshwork ready
sa-6r27  Write the spill postmortem
sa-pdk6  Reproduce the 600M-row spill cliff

$ meshwork why sa-vj0d
sa-vj0d blocked by 1:
- sa-pdk6 (open) — verify: test -f repro.log
```

Try to close something that isn't actually done, and the tool declines. This is the load-bearing rule: a task's `verify:` command must exit 0, observed, right now.

```
$ meshwork close sa-6r27
meshwork: sa-6r27 stays open: verify exit 1 (`test -f docs/postmortem.md`)

$ meshwork start sa-pdk6
$ meshwork comment sa-pdk6 --as claude "cliff reproduces at batch=64k; tracks the governor wakeup interval, not batch size"
$ touch repro.log        # stand-in for the actual work
$ meshwork close sa-pdk6
sa-pdk6 doing→done (verify exit 0)
```

(`close --waive "reason"` exists for the genuinely unverifiable. It's recorded and queryable — `WHERE waived IS NOT NULL` — so waiving is loud, not a loophole.)

And the part that kills the HANDOFF.md ritual: the next session doesn't read files to catch up. It gets `prime` — injected automatically by a SessionStart hook — which materializes the handoff from the store:

```
$ meshwork prime
sazed — 2 open, 1 done
engine/spill 1 · docs 1
next → sa-vj0d Fix spill batch sizing
  » Cliff is governor wakeup, not batch size — don't burn a session re-deriving
  » that (comment on sa-pdk6 has the repro). Try wakeup=250ms before touching
  » batch math.
  [engine/spill]
  verify: cargo test spill::batch
also ready (1 more, top 1):
- sa-6r27 Write the spill postmortem
recently done:
- 2026-08-06 sa-pdk6 Reproduce the 600M-row spill cliff
```

Everything in that digest is derived from the task files — counts, the category rollup, what's next and why, what just finished — except one thing: the `»` lines. That's the `handoff:` block, the outgoing session's voice to the incoming one, and it's the only authored part of the handoff. It lives on whichever task is up next, gets rewritten freely (history belongs in comments), and lint warns if you leave one on a task you close. Current conditions are always computed; commentary is always signed off by whoever knew something.

The digest is capped at 6KB ≈ 1.5K tokens — versus the 22K-token ritual it replaces — and when a store is big enough to threaten the cap, the truncation is explicit, never silent.

---

## Files are the API

A task is one markdown file. This is the entire storage format — the file above, verbatim:

```markdown
---
id: sa-vj0d
title: Fix spill batch sizing
status: open
category: engine/spill
needs: [sa-pdk6]
verify: cargo test spill::batch
handoff: |
  Cliff is governor wakeup, not batch size — don't burn a session re-deriving
  that (comment on sa-pdk6 has the repro). Try wakeup=250ms before touching
  batch math.
seq: 10
created: 2026-08-06
---

## log
- 2026-08-06 created
```

Hand-editing is legal and expected — fields without CLI flags (`seq:`, `docs:`, `handoff:`) are set with your editor, and `meshwork lint` validates the result (schema, cycles, dangling edges, post-merge damage; `lint --fix` repairs the mechanical cases). A file that fails to parse isn't dropped — it shows up as an `invalid` row in every listing until someone fixes it, because a task that silently vanishes is worse than a loud one.

Because tasks are files in git, concurrency is git's problem, which is a solved one: two sessions in separate worktrees can create tasks, comment on the *same* task, and close tasks, then merge without manual conflict resolution. The one merge artifact git can produce (a duplicated frontmatter key from union-merge) is exactly what `lint --fix` repairs.

The `seq` field is the priority primitive: integers with gaps of 10, lower runs sooner. There is no priority enum, no due date, no story points. If you need something ordered, order it.

For anything the canned verbs don't answer, the store is a SQL table:

```
$ meshwork q "SELECT category, count(*) AS n FROM tasks WHERE status='open' GROUP BY category ORDER BY n DESC"
category | n
docs | 1
engine/spill | 1
(2 rows)
```

Every verb also takes `--json` with a stable schema, for scripts and agents.

---

## Boundaries, deliberately

- **Zero network.** No verb touches the network today. (A one-way, append-only GitHub mirror — issues created, comments appended, nothing ever edited or closed remotely — is designed and queued, along with cross-repo dependencies and a portfolio-wide `ready`. This repo tracks that work in its own store.)
- **Never installs git hooks, never writes outside the repo.** The SessionStart hook that injects `prime` is Claude Code configuration you add yourself, once.
- **The CLI surface is frozen.** Anything not in the design doc's verb table is a non-goal, enforced by a test that diffs `--help` against the spec. Feature ideas default to the rejection list; that's the anti-Jira fence, and it's why the binary will still be small next year.
- **meshwork tracks meshwork.** This repo's own `meshwork/` store holds its remaining roadmap, the repo's gate runs `lint` + `prime` against it on every push, and the digest you get when you open a session here is the one described above. The measured claims in this README are from the requirements doc's evidence section; the migration that produced them is the current pilot.

---

## Getting it

Each consuming repo pins its own version — there is deliberately no global install:

```bash
echo "v0.1.0" > .meshwork-version     # commit this

VER=$(cat .meshwork-version)
DEST=~/.meshwork/versions/$VER
mkdir -p "$DEST"
gh release download "$VER" -R jbrjake/meshwork \
  -p "*aarch64-apple-darwin.tar.gz" -O - | tar -xz -C "$DEST"
"$DEST/meshwork" --help
```

Hooks and scripts invoke `~/.meshwork/versions/$(cat .meshwork-version)/meshwork`, so two repos can disagree about versions forever and neither can break the other.

If you drive repos with Claude Code, install the bundled skill — it teaches sessions the loop above, including the handoff ritual: see [`skill/references/install.md`](skill/references/install.md). Releases are darwin arm64 today; Linux and Windows builds are queued in the store.

Built on [DataFusion](https://datafusion.apache.org/). MIT — see [LICENSE](LICENSE).
