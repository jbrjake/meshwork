---
id: mw-1z1xsbw
title: Migrate this repo's own deploy to the plugin install
status: doing
category: meta/distribution
discovered-from: mw-nx91erh
verify: "all(exists docs/meshwork/meshwork, absent meshwork)"
docs:
  - .claude/skills/meshwork/references/migrate.md
seq: 250
created: 2026-08-28T13:59Z
claimed-by: claude (session_016iEafFdzwyKAtsU3AEMhaU)
---

Own-repo variant of references/migrate.md, with the dev-repo deviations
deliberate: the root ./meshwork shim (execs target/debug/meshwork, carries
the MESHWORK_AUTHOR block) moves to docs/meshwork/meshwork with its
relative path fixed; SessionStart repoints at the shim, keeping the
cargo-run fallback for fresh clones; permission rules sweep from
./meshwork and target/debug paths to the shim path; CLAUDE.md's session
ritual line follows the shim.

Kept on purpose, unlike an adopter migration: .claude/skills/meshwork/ is
the plugin's canonical SOURCE (install.md names it), not a vendored copy —
it stays. No .meshwork-version pin — the dev repo self-hosts on HEAD, so
the shim keeps exec'ing the debug build. The enabledPlugins entry
/plugin install wrote into .claude/settings.json commits with the
migration. sazed (mw-rtt16df) and leras (mw-mcx59sd) migrate separately,
each from its own session.

## log
- 2026-08-28T13:59Z created
- 2026-08-28T14:00Z open→doing — claimed by claude (session_016iEafFdzwyKAtsU3AEMhaU)
