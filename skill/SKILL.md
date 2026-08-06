---
name: meshwork
description: Use meshwork — the portfolio task tracker (markdown task files in git, SQL queries, no database). Use when a repo has a meshwork/ directory, when the session-start context shows a "meshwork — N open" digest, when migrating a repo off TODO.md/HANDOFF.md, or when asked to manage tasks with meshwork.
---

# meshwork

Task graph as markdown-with-frontmatter files under `meshwork/tasks/`, one file
per task. Single Rust binary, zero config, zero network. `meshwork` below means
the repo's pinned binary: `~/.meshwork/versions/$(cat .meshwork-version)/meshwork`.

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
- Status via verbs: `start`, `block --reason`, `reopen`, `drop`. Close ONLY via
  `meshwork close <id>` — it runs the task's `verify:` and closes on exit 0;
  `--waive "reason"` is the loud escape hatch.
- Notes: `comment <id> --as <author> "text"`. Files: `attach <id> <path>`.
- Session end: refresh the `handoff:` frontmatter block (your voice to the next
  session) on whatever task is up next — hand-edit the file, there is no verb.
  Never leave `handoff:` on a task you close (lint warns: handoff-stale).
  Anything history-worthy goes in a comment instead.

## Rules

- Task files are plain markdown — hand-edits are legal; run `meshwork lint`
  afterward (`lint --fix` repairs mechanical damage). Fields `add` has no flag
  for (`seq:`, `docs:`, `handoff:`) are set by editing the file — that is the
  intended path, not a workaround.
- `seq` is the priority primitive (integers, gaps of 10; lower = sooner). There
  is no priority field and no due date, deliberately.
- Every task should carry a `verify:` command (lint warns when missing) and
  `docs:` links (`path#§-anchor`) tying it to requirements/design sections.
- The CLI surface is frozen by design. If a verb doesn't exist, it's a
  deliberate non-goal — don't script around it; raise it with the owner.
- meshwork never touches the network (GitHub mirroring is a future explicit
  opt-in), never mutates GitHub, never installs git hooks.
