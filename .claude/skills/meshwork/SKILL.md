---
name: meshwork
description: Use meshwork — the portfolio task tracker (markdown task files in git, SQL queries, no database). Use when a repo has a meshwork/ directory, when the session-start context shows a "meshwork — N open" digest, when migrating a repo off TODO.md/HANDOFF.md, or when asked to manage tasks with meshwork.
---

# meshwork

Task graph as markdown-with-frontmatter files under `docs/meshwork/`, one file
per task. Single Rust binary, zero config, zero network. `meshwork` below means
the repo's committed `./meshwork` shim, which execs the pinned binary
(`~/.meshwork/versions/$(cat .meshwork-version)/meshwork` — install.md).

**Installing the binary or adopting meshwork in a new repo?** Read
`references/install.md` (pinned install, no globals) and `references/adopt.md`
(migration ritual) from this skill's directory first. Don't improvise either.

## Session ritual

- Session start: the SessionStart hook injects `meshwork prime` — the
  materialized handoff (counts + rollup, weather, the next task led by its
  `handoff:` commentary, also-ready, recent dones). Do not re-read TODO/HANDOFF
  files — the store is the worklist.
- `meshwork ready` → next actionable. `show <id>` full task. `why <id>` blocker
  frontier. `blocked`, `tree <id>` as needed. Raw SQL: `q "SELECT …" [--json]`.
- New work discovered mid-session: `meshwork add "title" --verify 'cmd'` — file
  it immediately, never carry it in your head, never append to a TODO.md.
- Terminal tasks auto-archive to `docs/meshwork/archive/` on close/drop
  (reopen moves them back). They stay fully queryable — never re-create or
  hand-move them; `lint --fix` repairs misplacements.
- Status via verbs: `start [--as <author>]`, `block --reason`, `reopen`,
  `drop`. Mirror the task you `start` into your harness's todo/console
  surface so the human can watch progress — the store stays the record.
  `start` claims the task for you (`claimed-by:`, advisory; author
  resolves `--as`, then `$MESHWORK_AUTHOR`, then config `default_author`);
  close/drop/reopen release the claim. Respect others' `[claimed: …]`
  annotations in prime/ready — pick unclaimed work. Close ONLY via
  `meshwork close <id>` — it runs the task's `verify:` and closes on exit 0;
  `--waive "reason"` is the loud escape hatch.
- Notes: `comment <id> "text"` (`@file`/`-` for long prose). Files:
  `attach <id> <path>`. The shim supplies the agent session's author;
  `default_author` stays the human's — outside the shim pass `--as`.
- Session end: refresh the `handoff:` block (your voice to the next session)
  on whatever task is up next — `meshwork set <id> --handoff "…"`
  (hand-editing the file works too). A handoff is an implementation
  brief, not a summary: name the files and symbols, state what is proven
  and what remains — the next session must not re-derive what this one
  learned. Never leave `handoff:` on a task you close (lint warns:
  handoff-stale). Anything history-worthy goes in a comment instead.

## Rules

- Task files are plain markdown — hand-edits are legal; run `meshwork lint`
  afterward (`lint --fix` repairs mechanical damage). Every field also has a
  CLI path: flags on `add` at creation (including `--seq`/`--docs`), then
  `meshwork set <id> --seq/--docs/--handoff`.
- Body prose goes ABOVE the tail sections — `## log` and `## comments` end
  the file; never append prose after them. A task that needs a real body at
  creation is a one-document `add --batch -`, not a hand-written file.
- `seq` is the priority primitive (integers, gaps of 10; lower = sooner). There
  is no priority field and no due date, deliberately.
- Every task should carry a `verify:` command (lint warns when missing) and
  `docs:` links (`path#§-anchor`) tying it to requirements/design sections.
- Author tasks as work orders. The title is an imperative action ("Fix the
  door check"), never a finding or a status — a finding-shaped title hides
  the fix it implies. The `verify:` must FAIL while the work is undone; a
  verify that already passes proves nothing about the work.
- The CLI surface is frozen by design. If a verb doesn't exist, it's a
  deliberate non-goal — don't script around it; raise it with the owner.
- meshwork never touches the network (GitHub mirroring is a future explicit
  opt-in), never mutates GitHub, never installs git hooks.
