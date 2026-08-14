# Adopting meshwork in a repo (migration ritual — read only when migrating)

Prerequisite: the pinned binary is installed and the `./meshwork` shim is
committed (`install.md`). Every `meshwork` below means the shim.

1. Hunt, don't assume. The old ritual rarely lives only at `./TODO.md` —
   handoffs hide under `docs/`, gate scripts call the old checker. Build
   the retirement list repo-wide first:

   ```sh
   git grep -ilE 'todo\.md|handoff|check-todo'
   ```

   Expect hits well beyond the files themselves: README, CLAUDE.md,
   baseline docs, and gate scripts (smoke/regression/file-length checks
   often invoke check-todo.sh).
2. `./meshwork init` — creates `docs/meshwork/` + config. It never installs git hooks
   and never writes outside the repo.
3. For each TODO found: `./meshwork import todo <path>` — checkboxes
   become task files. import absorbs all prose between checkboxes into the
   preceding task's body — a section-structured TODO turns whole ledgers
   into one task's body. Triage every generated body: split section prose
   into its own tasks. Review every generated file before committing
   (import is a one-shot migration, not a sync). Then `./meshwork lint`.
4. Wire the session-start digest into the repo's `.claude/settings.json`
   (merge with existing settings — never replace the file):

   ```json
   {
     "hooks": {
       "SessionStart": [
         { "hooks": [ { "type": "command",
             "command": "\"$CLAUDE_PROJECT_DIR\"/meshwork prime 2>/dev/null || true",
             "timeout": 30, "statusMessage": "meshwork prime" } ] }
       ]
     }
   }
   ```

   Prove it fired: `claude -p "Without tools: quote the first line the
   session-start hook injected"` — expect the `meshwork — N open` digest.
5. Retire the old ritual **in the same commit**, working the step-1 list to
   zero: delete TODO.md (its content now lives in `docs/meshwork/`), delete
   check-todo.sh and every reference to it, and DELETE HANDOFF.md outright —
   `prime` is the handoff (meshwork DESIGN §7b). Two task systems is worse
   than one; a hand-written handoff is a second one.
6. Last of all, recast and red-check the migrated verifies — after the
   retirement commit, not before. Recast each into the DSL where it fits
   (`run cargo test <filter>`, `exists <path>`, `contains <path>
   <lit|/regex/>`, `all(p, …)`): DSL skips the per-clone approval gate
   while the task's history is store-only, and `run cargo test` natively
   refuses a vacuous pass. Red-check: `start` runs DSL checks itself and
   warns "already green". For verifies that must stay shell, run
   `sh -c '<verify>'` (close's shell). Exit 0 means the migration itself
   satisfied it — e.g. a grep of the archive now matches the task's own
   migrated prose — so it detects nothing; rewrite it. Exit 127 means it
   can't run under close (agent-shell functions like `rg` don't exist
   there); recast it in grep/test/cargo. Only a verify that runs and
   still fails is armed.
