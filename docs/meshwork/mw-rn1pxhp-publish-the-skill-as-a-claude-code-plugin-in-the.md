---
id: mw-rn1pxhp
title: Publish the skill as a Claude Code plugin in the owner's marketplace
status: open
category: skill
seq: 72
verify: grep -q '"name"' .claude-plugin/plugin.json
docs:
  - .claude/skills/meshwork/references/install.md
created: 2026-08-12T20:01Z
---
Owner directive (2026-08-12): make the skill installable the easy way — a
`/plugin install`-able entry in the owner's Claude plugin marketplace —
instead of the manual vendor-from-release-tarball ritual.

Found state: no marketplace repo exists under ~/Documents/code as of
2026-08-12 (no `*/.claude-plugin/marketplace.json` anywhere); locating or
creating the marketplace repo is in scope.

Deliverable: a `.claude-plugin/plugin.json` in this repo exposing the
skill as a plugin, plus a marketplace entry pointing at it. Close only
once the marketplace entry exists too — the verify can only see the
manifest half (no network).

Doctrine to reconcile in install.md: the per-repo ruling (binary AND
skill pinned per-repo from the release, nothing global). A marketplace
install is user-scoped, so the plugin distributes the skill surface
only, never the binary, and the skill must keep deferring to each repo's
pinned shim/version when present.

## log
- 2026-08-12T20:01Z created
