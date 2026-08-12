---
id: mw-rn1pxhp
title: Publish the skill as a Claude Code plugin in the owner's marketplace
status: done
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
- 2026-08-12T23:29Z open→doing — claimed by claude (session_016iEafFdzwyKAtsU3AEMhaU)
- 2026-08-12T23:34Z doing→done — verify exit 0 @ 8423057+8

## comments
- 2026-08-12T23:34Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Landed: .claude-plugin/plugin.json with skills: ./.claude/skills/ (validator passed; headless --plugin-dir probe lists meshwork:meshwork — skill served from the custom path, no restructure). meshwork entry appended to the jbrjake marketplace plugins list. README: marketplace-only install block above quick-start; --plugin-dir clone variant folded into getting-it. install.md ruling reconciled: binary never global, plugin is the one sanctioned user-scoped skill path, ships skill surface only, defers to each repo's shim-pinned version; vendored per-repo skill copy stays for repos pinning the skill text itself.
