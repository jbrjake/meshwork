---
id: mw-y5zmkxb
title: Create the three public notes-demo repos and give each a meshwork store with its alias
category: meta/demo
seq: 225
verify: grep -q 'alias = "nt"' ../meshwork-demo-notes-cli/docs/meshwork/config.toml && grep -q 'alias = "sy"' ../meshwork-demo-notes-sync/docs/meshwork/config.toml && grep -q 'alias = "pf"' ../meshwork-demo-notes-portfolio/docs/meshwork/config.toml && git -C ../meshwork-demo-notes-cli rev-parse -q --verify origin/main && git -C ../meshwork-demo-notes-sync rev-parse -q --verify origin/main && git -C ../meshwork-demo-notes-portfolio rev-parse -q --verify origin/main
docs:
  - docs/PLAN-demo-notes.md#settled-decisions
status: done
created: 2026-10-02T15:21Z
---
Create `jbrjake/meshwork-demo-notes-cli`, `-sync` and `-portfolio` on GitHub, public. Creating them publishes under the owner's account, so confirm with the owner in the session that runs `gh repo create`. The plan and this task are not that confirmation.

For each repo:
- Clone it to ~/Documents/code/<name>, the demo registry's default local path, and `mdutil -i off` the checkout.
- `meshwork init` it per the skill's install reference.
- Set `alias` in docs/meshwork/config.toml before the first `add`: cli `nt`, sync `sy`, portfolio `pf`. `init` would derive `me` for all three.
- Commit with Conventional Commits and push main.

From the first commit, task files ride in store-only commits, never with code and never with a `.meshwork-version` rewrite. Provenance judges every commit that ever touched a task file, and a mixed one gates that task's `run` verifies for good. Nobody is present at the recording to approve them.

The verify reads the sibling checkouts because the DSL is confined to this repo. It checks that each config carries its alias and that each checkout has pushed main.

## log
- 2026-10-02T15:21Z created
- 2026-10-02T16:48Z open→doing — claimed by claude (6fa0dcc3-62c5-4a7b-a6b1-de25f58287de)
- 2026-10-02T17:10Z doing→done — verify exit 0 @ e9c1456+2

## comments
- 2026-10-02T17:09Z [claude (602c381b-d7db-491e-8df6-85682e6152ed)] Owner confirmed in session 2026-10-02 (session ): create jbrjake/meshwork-demo-notes-cli and -portfolio as public repos (sync already existed), and push commits and tags to all three demo repos as each day 0 is built.
- 2026-10-02T17:10Z [claude (602c381b-d7db-491e-8df6-85682e6152ed)] Done: all three repos are public, cloned at ~/Documents/code/<name>, pinned to v0.5.2, aliased sy/nt/pf, and main is pushed (sync e8f077d, cli 930f832, portfolio 8204bf8). v0.5.2's init writes no shim, so each store got the shim from v0.5.2's install reference, which is byte-identical to hooks/meshwork. The body's 'mdutil -i off the checkout' step cannot work: mdutil works per volume, not per directory. Per the plan's Risks section, each crate's gate re-creates target/.metadata_never_index instead.
