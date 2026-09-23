---
name: meshwork
description: Use meshwork — the portfolio task tracker (markdown task files in git, SQL queries, no database). Use when a repo has a meshwork/ directory, when the session-start context shows a "meshwork — N open" digest, when migrating a repo off TODO.md/HANDOFF.md, or when asked to manage tasks with meshwork.
---

# meshwork

Task graph as markdown-with-frontmatter files under `docs/meshwork/`, one file
per task. Single Rust binary, zero config, zero network. `meshwork` below means
the repo's committed shim, `docs/meshwork/meshwork`, which execs the pinned
binary. Installing, adopting, or upgrading a legacy deploy: read
`references/install.md`, `references/adopt.md`, or `references/migrate.md` —
don't improvise any of them.

## Four rules

- A spoken instruction becomes a task before the work starts; the answer to
  "file it" is the id, not the prose.
- Never attribute a request or ruling to the owner unless it is in this
  session's transcript — a store comment is not a ruling, nor is a handoff.
- Owner-scoped fields (asset picks, rulings, licenses) surface as a conflict
  for the owner; never resolve one by editing the task.
- Memory files are yours; the store is the team's. A durable fact that lives
  only in memory or in the harness todo is untracked.

## Session ritual

- Session start: the SessionStart hook injects `meshwork prime` — counts,
  weather, the inbox, the next task led by its `handoff:`, also-ready, recent
  dones. The store is the worklist; there is no TODO/HANDOFF file to read.
  Re-run `prime` when the question changes; load this skill before filing.
- `ready` → the queue. `show <id>`; `why <id>` (open-blocker frontier, or an
  umbrella's `hidden: N live children`); `blocked`; `tree <id>`. Prior art
  before filing: `search <term>` — literal, case-insensitive, over titles,
  bodies, handoffs, comments and log notes, archives included; never
  `grep -r` the store. Raw SQL: `q "SELECT …" [--json]` — `q --help` lists
  the six tables' columns and the thirteen views; `--json` puts rows under
  `data.rows`.
- Status via verbs: `start` (claims it — `claimed-by:`, advisory; respect
  others' `[claimed: …]`), `block --reason`, `reopen`, `drop`. Mirror the
  task you start into the harness todo so the human can watch; the store
  stays the record. Close ONLY via `close <id>` — it runs `verify:` and
  closes on exit 0; `--waive "reason"` is the loud escape hatch. Terminal
  tasks auto-archive to `archive/` and stay queryable; never hand-move them.
- Notes: `comment <id> "text"` (`@file`/`-` for prose); files: `attach`.
  The shim supplies the session author; outside it pass `--as`.
- Session end: `set <id> --handoff @file` on whatever is up next — an
  implementation brief in your voice: files, symbols, what is proven, what
  remains. Never leave `handoff:` on a task you close; history is a comment.

## The inbox and sibling stores

- An ask is a task in YOUR store carrying `to: <repo>`; nothing is sent. It
  surfaces in that repo's `prime`/`ready` until a task carrying
  `answers: <its gid>` is done; a live answer shows as `answered-by`. Your
  own asks leave your `ready` for its `asks out` line. The whole inbox,
  open answers included, is one query over the `asks` view:
  `portfolio q "SELECT gid, title, age_h, answer_gid, answer_status FROM asks WHERE to_repo = '<me>' AND unanswered"`.
  A per-repo `q … WHERE addressed_to = '<me>'` is your OUTBOUND asks, not
  the inbox.
- A sibling's id resolves only in its own store, through ITS shim —
  `(cd ../<repo> && docs/meshwork/meshwork show <id>)` — versions pin per
  repo and the shim supplies the session author; never guess its file path.
  The union verbs are `portfolio ready` / `next` / `q`; register repos once
  in the portfolio's `repos.toml`.
- `--to`, `--answers` and `--relates` ride `add` and `set`; several tasks,
  or handles, go in an `add --batch -` document (`--dry-run` prints it):
  ```
  ---
  title: <imperative>
  to: leras
  verify: <predicate>
  ---
  <body>
  ```

## Authoring

- Lead with the flags: `add "title" --body @file --docs path#§-anchor --seq N
  --cat a/b --verify '<predicate>'`; several tasks, or structured
  frontmatter, is one `add --batch -` document — `id:` omitted, a local
  `handle:` usable as `@handle` in needs/parent/from/relates, values as
  plain as the flags take them (a `: ` inside needs no quoting), atomic:
  all files or none. Later: `set <id> --seq/--handoff/--verify/--cat/--title/
  --body/--parent/--from/--relates/--to/--answers`, `--docs <link>` to
  append or `--docs <old> <new>` to replace; edges: `dep add <a> --needs <b>`.
- Titles are imperative work orders ("Fix the door check"), never a finding.
  Every task carries `verify:` and `docs:` (`path#§-anchor`); lint warns.
- Hand-edits are legal, then `lint` (`--fix` mends mechanical damage;
  `--explain <code>` prints one code's rows alone, a summarized finding
  unfolded, a heuristic's note first) — but a block value with blank lines inside (`handoff: |`, list
  keys) is replaced whole or via `set`, never in part, and nothing goes
  below `## log` / `## comments`. Reproducible transcripts:
  `MESHWORK_ID_SEED=<n>` (deterministic ids), `MESHWORK_TODAY=YYYY-MM-DD`
  (the clock).
- How tasks mesh: `--parent` = umbrella, hidden from `ready` while a child
  lives; `--needs` (`repo#id` crosses repos) = hard order, gates `ready`;
  `relates:` = soft; `--from` = provenance. Priority is graph then `seq`
  (integers, gaps of 10, lower sooner) — no priority field, no due date.

## Verifies

The close gate is a DSL — one predicate, or `all(p, p, …)` — never shell:
```
exists <path>                  absent <path>     (exists: one * in the last segment)
contains <path> <literal>      contains <path> /<regex>/
lacks <path> <literal|/regex/>     the file exists and does not match; missing refuses
run cargo test|build|fmt <args…>   argv-spawned; args carry no leading dash
                                   (letters digits _ . : / = -); package=<crate>
                                   and target=<name> spell -p and --test
all(<pred>, <pred>, …)
```
Paths are repo-relative, no `..`. `run cargo test` must observe `ok. N
passed`, N ≥ 1, and runs approval-free while the task's git history is
store-only — commit task files apart from code. Text not keyword-led is
legacy shell: gated per clone, lint warns `verify-shell`.

- A verify must FAIL while the work is undone; `start` red-checks it, and
  "already green" means it cannot detect the work (`verify <id>` runs it
  any time, closing nothing). Non-code shapes: an umbrella closes on its
  live-children count (`q "SELECT live_children FROM graph WHERE id =
  '<id>'" --json`, `"rows":[[0]]`); an owner-gated hold on a hand-written
  dated marker, `contains <task-file> /2026-09-01 owner approved/` —
  date-first, so CLI stamps never match; an artifact task on `exists
  <path>`, the verify naming the deliverable — `all(exists <path>,
  contains <path> …)` when an empty stub must not pass; the `exists` arm
  tells lint the file is the work, so `verify-path-missing` stays quiet
  until it appears.
- A `contains` regex is grep-like: `^`/`$` anchor lines and `.` stops at
  a newline, so a two-phrase `.*` pattern misses a marker that wrapped.
  Prefer a one-line marker; a pattern that must span a wrap leads with
  `(?s)`.
- Traps: a grep the task's own file or an archive already satisfies; for
  legacy shell, piped tails report the tail's exit, and close's `sh -c` has
  no agent-shell functions like `rg`.

## Boundaries

The CLI surface is frozen; a missing verb is a deliberate non-goal — raise it
with the owner, never script around it. meshwork never touches the network,
never mutates GitHub, never installs git hooks.
