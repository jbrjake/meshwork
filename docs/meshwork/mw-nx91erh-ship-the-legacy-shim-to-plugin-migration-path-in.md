---
id: mw-nx91erh
title: Ship the legacy-shim-to-plugin migration path in the next release
status: open
category: meta/distribution
discovered-from: mw-84h1mve
verify: exists .claude/skills/meshwork/references/migrate.md
seq: 250
created: 2026-08-24T13:09Z
---

Owner ruling 2026-08-24 (answering mw-84h1mve's shim-vs-hook question):
neither bless the hook nor hand-install the shim — meshwork as a Claude
plugin must be able to migrate legacy shim deploys to modern plugin
installs, and that migration path ships in the next release. Then every
adopter repo upgrades to the latest version and installs through Claude
Code plugins. sazed and leras pre-date the plugin (mw-rn1pxhp) and were
set up manually by an agent — they are the first migration targets.

The plugin half exists: .claude-plugin/plugin.json serves the skill, and
marketplace installs resolve the newest tag, so cutting a release IS the
publish. What is missing is the migration ritual: what a legacy repo
removes (vendored skill copy, hook-built binary paths), what it gains
(plugin install, committed shim per install.md with MESHWORK_AUTHOR
session tagging), and the ordering. The verify names the deliverable —
references/migrate.md in the skill. install.md/adopt.md updates ride
this task. The shim-vs-hook evidence on mw-84h1mve (zero --as in 35
shim-less sessions, three distinct $(cat .meshwork-version) failures)
should shape the modern endpoint; adopt.md also gains the warning
against version-pinned permission rules. When this lands: file the
per-repo migration sweep (sazed and leras first, to: asks per repo).

## log
- 2026-08-24T13:09Z created
