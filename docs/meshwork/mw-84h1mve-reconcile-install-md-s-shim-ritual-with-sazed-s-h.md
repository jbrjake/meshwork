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

## comments
- 2026-08-12T20:50Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] The shim-vs-hook divergence has a measured cost now. sazed ran the whole week shim-less: zero --as uses in 35 sessions, so all 8 agent comments and every claimed-by stamp read as the owner in prime's weather (fc237a1a, 4b5a9264). The $(cat .meshwork-version) incantation failed three distinct ways: wrong-cwd cat (63b829ba), a sandbox EPERM retried 4x that killed every meshwork verb for a session (632ce3d2), and a version-pinned settings.local.json allow-rule that rotted at the v0.2.0 upgrade (f6e7cfbc). A committed shim fixes attribution and all three fragilities at once; adopt.md should also warn against version-pinned permission rules.
