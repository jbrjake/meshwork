---
id: mw-mcx59sd
title: Migrate leras's meshwork deploy to the plugin install
category: meta/distribution
discovered-from: mw-nx91erh
needs: ["mw-h4s4gka"]
to: leras
seq: 256
verify: test -x ../leras/docs/meshwork/meshwork && test ! -e ../leras/meshwork && test ! -d ../leras/.claude/skills/meshwork
status: done
created: 2026-08-24T13:32Z
---
leras's legacy shape, observed 2026-08-24: pin v0.2.1; root shim
./meshwork (already carries the MESHWORK_AUTHOR block; resolves
$(dirname "$0")/.meshwork-version); SessionStart targets
"$CLAUDE_PROJECT_DIR"/meshwork; vendored skill copy at
.claude/skills/meshwork/.

references/migrate.md ritual, root-shim variant: plugin install → pin bump
onto the release from the needs-task → git mv meshwork
docs/meshwork/meshwork and point its cat at ../../.meshwork-version →
repoint the hook at docs/meshwork/meshwork → permission-rule sweep →
git rm -r .claude/skills/meshwork → one commit; prove the author block
survived the move (comment via the shim, expect claude (<session-id>)).

## log
- 2026-08-24T13:32Z created
- 2026-08-28T14:46Z open→doing — claimed by claude (session_016iEafFdzwyKAtsU3AEMhaU)
- 2026-08-28T14:53Z doing→done — verify exit 0 @ ea3479d+4

## comments
- 2026-08-24T13:35Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Owner instruction 2026-08-24: the ritual includes recasting the store's pre-DSL shell verifies into the verify DSL (migrate.md step 6, pointing at adopt.md step 6 for recast-and-red-check). Under the bumped pin, shell verify text this clone did not author prompts per-clone before it runs. Land the recasts as their own store-only commit — mixing task files with config/code edits costs run-cargo-test its approval-free path.
