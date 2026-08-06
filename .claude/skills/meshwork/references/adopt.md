# Adopting meshwork in a repo (migration ritual — read only when migrating)

Prerequisite: the pinned binary is installed (`install.md`). Every `meshwork`
below means that pinned binary.

1. `meshwork init` — creates `docs/meshwork/` + config. It never installs git hooks
   and never writes outside the repo.
2. If the repo has a TODO.md: `meshwork import todo TODO.md` — checkboxes
   become task files. Review every generated file before committing (import is
   a one-shot migration, not a sync). Then `meshwork lint`.
3. Wire the session-start digest into the repo's `.claude/settings.json`
   (merge with existing settings — never replace the file):

   ```json
   {
     "hooks": {
       "SessionStart": [
         { "hooks": [ { "type": "command",
             "command": "~/.meshwork/versions/$(cat \"$CLAUDE_PROJECT_DIR\"/.meshwork-version)/meshwork prime 2>/dev/null || true",
             "timeout": 30, "statusMessage": "meshwork prime" } ] }
       ]
     }
   }
   ```

   Prove it fired: `claude -p "Without tools: quote the first line the
   session-start hook injected"` — expect the `meshwork — N open` digest.
4. Retire the old ritual **in the same commit**: delete TODO.md (its content
   now lives in `docs/meshwork/`), delete check-todo.sh and any references to it,
   and DELETE HANDOFF.md outright — `prime` is the handoff (meshwork DESIGN
   §7b). Two task systems is worse than one; a hand-written handoff is a
   second one.
