---
id: mw-rtt16df
title: Migrate sazed's meshwork deploy to the plugin install
category: meta/distribution
discovered-from: mw-nx91erh
needs: ["mw-h4s4gka"]
to: sazed
seq: 254
verify: test -x ../sazed/docs/meshwork/meshwork && test ! -d ../sazed/.claude/skills/meshwork
status: open
created: 2026-08-24T13:32Z
---
sazed's legacy shape, observed 2026-08-24: pin v0.2.0; no shim anywhere
(neither ./meshwork nor docs/meshwork/meshwork); the SessionStart hook
rebuilds ~/.meshwork/versions/$(cat "$CLAUDE_PROJECT_DIR"/.meshwork-version)/meshwork;
vendored skill copy at .claude/skills/meshwork/. Nothing sets
MESHWORK_AUTHOR — the measured cost (zero --as across 35 sessions, three
distinct $(cat) failure modes) is mw-84h1mve's 2026-08-12 evidence comment
and all of it came from this repo.

Run the skill's references/migrate.md ritual, hook-only variant: plugin
install → pin bump onto the release from the needs-task → create the shim
fresh at docs/meshwork/meshwork → repoint SessionStart at the shim → sweep
.claude/settings.json and settings.local.json for version-pinned
allow-rules (one already rotted at the v0.2.0 upgrade) → git rm -r
.claude/skills/meshwork → one commit, then the attribution probe from
migrate.md step 7. Work it from a sazed session; live sessions share that
repo, so check what is staged and commit with explicit pathspec.

## log
- 2026-08-24T13:32Z created

## comments
- 2026-08-24T13:35Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Owner instruction 2026-08-24: the ritual includes recasting the store's pre-DSL shell verifies into the verify DSL (migrate.md step 6, pointing at adopt.md step 6 for recast-and-red-check). Under the bumped pin, shell verify text this clone did not author prompts per-clone before it runs. Land the recasts as their own store-only commit — mixing task files with config/code edits costs run-cargo-test its approval-free path.
