# Adopting meshwork in a repo (TODO.md retirement — read only when adopting)

Prerequisite: the plugin is installed and the pinned binary is fetched
(`install.md`). Every `meshwork` below means the shim,
`docs/meshwork/meshwork`, which `init` writes — never create anything at
the adopter's repo root.

1. Hunt, don't assume. The old ritual rarely lives only at `./TODO.md` —
   handoffs hide under `docs/`, gate scripts call the old checker. Build
   the retirement list repo-wide first:

   ```sh
   git grep -ilE 'todo\.md|handoff|check-todo'
   ```

   Expect hits well beyond the files themselves: README, CLAUDE.md,
   baseline docs, and gate scripts (smoke/regression/file-length checks
   often invoke check-todo.sh).
2. `~/.meshwork/versions/"$(cat .meshwork-version)"/meshwork init` — the
   one call made before the shim exists. It fills in `docs/meshwork/` +
   config and writes the shim; it never installs git hooks and never writes
   outside the store. Commit `.meshwork-version` with `docs/meshwork/`.
3. For each TODO found: `docs/meshwork/meshwork import todo <path>` — checkboxes
   become task files. Prose indented under a checkbox lands in that task's
   body (a long block is absorbed whole, counted by id); prose at column
   zero that no checkbox owns — preambles, section notes, ledgers — carries
   into one triage task, counted. Triage every generated body and the
   triage task: split section prose into its own tasks. Review every
   generated file before committing (import is a one-shot migration, not a
   sync). Then `docs/meshwork/meshwork lint`.
4. Prove the session-start digest arrives. The plugin's SessionStart hook
   injects `prime` in every project that carries `docs/meshwork/` and
   `.meshwork-version` — nothing is added to the repo's
   `.claude/settings.json`. Start a session in the repo: the first thing in
   context is the `<repo> — N open` digest. Never add a per-repo prime hook
   there — the plugin's hook yields to one it finds, which leaves the repo
   on the old ritual.

   When permission allow-rules for meshwork get added, they target the
   committed shim path — never a `~/.meshwork/versions/<tag>/…` path. A
   rule that embeds a release tag dies silently at the next pin bump and
   every meshwork call starts prompting again (`migrate.md` step 5 is the
   repair).
5. Retire the old ritual **in the same commit**, working the step-1 list to
   zero: delete TODO.md (its content now lives in `docs/meshwork/`), delete
   check-todo.sh and every reference to it, and DELETE HANDOFF.md outright —
   `prime` is the handoff. Two task systems is worse than one; a
   hand-written handoff is a second one.
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
