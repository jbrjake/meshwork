---
id: mw-84h1mve
title: "Reconcile install.md's shim ritual with sazed's hook-based pin resolution"
category: meta/distribution
verify: grep -q SessionStart .claude/skills/meshwork/references/install.md || test -x ../sazed/meshwork
docs:
  - .claude/skills/meshwork/references/install.md
discovered-from: mw-bds8yq5
status: open
created: 2026-08-10T18:34Z
---
Found during the mw-bds8yq5 upgrade (2026-08-10): install.md prescribes a
committed ./meshwork shim as "what sessions actually run", but sazed — a
live consumer, now pinned to v0.2.0 — has no shim; its SessionStart hook
builds the raw ~/.meshwork/versions/$(cat .meshwork-version)/meshwork
path instead. Doc and reality disagree; owner picks the direction:
either bless the hook pattern in install.md as a documented alternative,
or install the shim in sazed per the existing ritual. Whichever wins:
the shim also carries the MESHWORK_AUTHOR session-tagging — a hook-based
repo must replicate that or agent comments silently fall through to
default_author. Check what sazed's hook actually exports before ruling.
install.md is canonical skill source here, so a doc change ships with
the next release.

## log
- 2026-08-10T18:34Z created
