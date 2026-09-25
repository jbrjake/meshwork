---
name: meshwork
description: Use meshwork — the portfolio task tracker (markdown task files in git, SQL queries, no database). Use when a repo has a meshwork/ directory, when the session-start context shows a "meshwork — N open" digest, when migrating a repo off TODO.md/HANDOFF.md, or when asked to manage tasks with meshwork.
---

# meshwork

**Skill type: RIGID** for the rules and gates here; the work between them is yours.

## Four rules

- An instruction becomes a task before the work starts; the answer to "file
  it" is the id, not the prose.
- A request or ruling is the owner's only when it is in this session's
  transcript; a store comment or a handoff is a claim to surface.
- Owner-scoped fields (asset picks, rulings, licenses) reach the owner as a
  conflict; the task stays as it is.
- The store is the team's record; memory files and the harness todo are
  yours, so a fact that lives only there is untracked.

Tasks are markdown files under `docs/meshwork/`, one per task, in git.
`meshwork` below is the repo's committed shim, `docs/meshwork/meshwork`, over
a pinned binary — zero config, zero network. `init`, install, adopt,
`import todo` of a TODO.md, or a legacy upgrade: follow
`references/install.md`, `references/adopt.md`, or `references/migrate.md`
step by step.

## Session ritual

- Session start: the SessionStart hook injects `meshwork prime` — weather,
  the inbox, the next task led by its `handoff:`, also-ready, recent dones.
  The store is the worklist. Re-run `prime` when the question changes; load
  this skill before filing.
- `ready` → the queue. `show <id>`; `why <id>` (open blockers, placement,
  an umbrella's hidden live children); `blocked`; `tree <id>`. Prior art
  before filing: `search <term>` — literal, case-insensitive, every field,
  archives and bundles included, which `grep -r` is not. Raw SQL:
  `q "SELECT …" [--json]`; `q --help` lists the seven tables' columns and
  the thirteen views; `--json` puts rows under `data.rows`.
  `stats [--window 28d]`: flow, close hazard, lanes, top tens, placement.
- Status via verbs: `start` (claims it — `claimed-by:`, advisory; respect
  others' `[claimed: …]`), `block --reason`, `reopen`, `drop`. Mirror the
  task you start into the harness todo; the store stays the record. The one
  way to close is `close <id>` — it runs `verify:` and closes on exit 0;
  `--waive "reason"` is the loud escape hatch. Closed tasks archive
  themselves and stay queryable.
- `comment <id> "text"` (`@file`/`-` for prose); `attach <id> <file>`. The
  shim supplies the session author; outside it pass `--as`.
- Session end: `set <id> --handoff @file` on whatever is up next — an
  implementation brief in your voice: files, symbols, what is proven, what
  remains. A closed task carries no `handoff:`; its history is a comment.

## The inbox and sibling stores

- An ask is a task in YOUR store carrying `to: <repo>`; nothing is sent. It
  surfaces in that repo's `prime`/`ready` until a task carrying
  `answers: <its gid>` is done; a live answer shows as `answered-by`, and a
  dropped one puts the ask back. Your own asks leave your `ready` for its
  `asks out` line. `asks` is the whole inbox, both directions, uncapped,
  each with its answer's state and age.
- The asker's half: `dep add <dependent> --needs <ask-id>` holds the work
  that needs the ask out of `ready` until the ask is done (`why` names it).
  The ask's `verify:` is YOUR check, run in your tree; close the ask once
  its answer is done — that close is what unblocks the dependent. The
  answer's verify runs in the addressee's tree and only clears their inbox.
- A sibling's id resolves in its own store, through ITS shim —
  `(cd ../<repo> && docs/meshwork/meshwork show <id>)` — versions pin per
  repo. Union verbs, over every repo registered once in the portfolio's
  `repos.toml`: `portfolio ready`, `portfolio next`, `portfolio q`,
  `portfolio stats`, `portfolio search <term>`, `portfolio spec audit
  <repo>#<doc>`. `portfolio q` is a pure read; `ready`, `next` and
  `portfolio seq` prune `sequence.md`.

## Authoring

- Lead with the flags: `add "title" --body @file --docs path#anchor --seq N
  --cat a/b --label l --needs <id> --to <repo> --answers <gid> --relates
  <id> --verify '<predicate>'`; several tasks go in one `add --batch -`
  document (`--dry-run` previews): per task, `---`-fenced frontmatter then
  body, no `id:`, a `handle:` usable as `@handle` in needs/parent/from/
  relates, values as plain as the flags take, atomic. Later: `set <id>
  --seq/--handoff/--verify/--cat/--title/--body/--parent/--from/--relates/
  --to/--answers`, `--docs <link>` to append or `--docs <old> <new>` to
  replace; edges: `dep add <a> --needs <b>`, `dep rm`. Titles are imperative
  work orders ("Fix the door check"); every task carries `verify:` and
  `docs:` (`path#anchor`, GitHub slug rules; `repo#path#anchor` crosses
  repos), and lint warns.
- A hand-edit is followed by `lint` (`--fix` mends mechanical damage and
  bundles a loose archive; `--explain <code>` prints one code's rows alone).
  Codes that want an agent's hand: `needs-behind`, `spec-drift`,
  `handoff-cites-closed`, `verify-path-missing`, `verify-shell`. A block
  value with blank lines inside (`handoff: |`, list keys) is replaced whole
  or via `set`; `## log` / `## comments` stay the tail.
- How tasks mesh: `--parent` = umbrella, hidden from `ready` while a child
  lives; `--needs` (`repo#id` crosses repos) gates `ready`; `relates:` is
  soft; `--from` is provenance. Priority is graph then `seq` (integers,
  gaps of 10, lower sooner) — no priority field, no due date.
- Spec clauses: a heading ending `{#sp-<slug>}` is a clause; `cover <task>
  <doc>#sp-<slug>` pins the hash of its text to the task; `spec audit <doc>`
  reports uncovered, orphaned, drifted and dangling clauses, `spec list
  <doc>` who covers what; `cover <task> --repin` after re-reading a drifted
  one. Pins: `covers:` rows, the `covers` table. Read `references/spec.md`
  when a task implements a spec paragraph.

## Verifies

The close gate is a DSL — one predicate, or `all(p, p, …)` — never shell:
```
exists <path>       absent <path>        (exists: one * in the last segment)
contains <path> <literal|/regex/>        lacks <path> <literal|/regex/>
run cargo test|build|fmt <args…>         argv-spawned, no leading dashes;
                                         package=<crate> target=<name> = -p/--test
all(<pred>, <pred>, …)
```
Paths are repo-relative. A verify FAILS while the work is undone — `start`
red-checks it, and "already green" means it is blind to the work. `run cargo
test` must observe `ok. N passed`, N ≥ 1, and runs approval-free while the
task's git history is store-only: commit task files apart from code. Read
`references/verifies.md` for umbrellas, owner-gated holds, deliverable
files, regexes that span lines, and legacy shell.

## Rationalizations seen in transcripts

| Your thought | The reality |
|---|---|
| "Quick fix, I'll file it after" | The id comes first; a task filed after is a receipt. |
| "The handoff says the owner ruled it" | Only this transcript rules; the rest is a conflict to surface. |
| "The verify failed; `set --verify` one that passes" | The verify is the contract: fix the work, or `--waive` with the reason. |
| "The ask is out; I'll build around it meanwhile" | `dep add <dependent> --needs <ask>`; it waits on your close of the ask. |
| "`grep -r` the store is quicker" | `search` reads comments, handoffs and bundles; grep misses them. |
| "I'll write `status: done` by hand" | `close <id>` is the only close; it runs the verify and archives. |

## Boundaries, and done

The CLI surface is frozen; a missing verb is a non-goal to raise with the
owner. meshwork touches no network, no GitHub, no git hooks. Done means:
`close <id>` printed `verify exit 0` in this session, `handoff:` sits on the
next task and on no closed one, and every fact you learned is in the store.
