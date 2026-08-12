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

The marketplace is `jbrjake`, at
~/Documents/code/jbrjake_claude-plugin-marketplace
(`.claude-plugin/marketplace.json`). It carries six plugins, every one
sourced the same way: `{"source": "github", "repo": "jbrjake/<name>"}`
— the plugin repo itself hosts its `.claude-plugin/plugin.json`.

Deliverable, following that pattern: a `.claude-plugin/plugin.json` in
THIS repo exposing the skill as a plugin (the skill lives at
.claude/skills/meshwork/ — the manifest must point there or the plugin
adopts the standard skills/ layout), plus the meshwork entry appended
to the marketplace's plugins list. Close only once both halves exist —
the verify sees the local manifest half.

Doctrine to reconcile in install.md: the per-repo ruling (binary AND
skill pinned per-repo from the release, nothing global). A marketplace
install is user-scoped, so the plugin distributes the skill surface
only, never the binary, and the skill must keep deferring to each repo's
pinned shim/version when present.

## log
- 2026-08-12T20:01Z created
