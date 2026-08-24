---
id: mw-84h1mve
title: "Reconcile install.md's shim ritual with sazed's hook-based pin resolution"
category: meta/distribution
verify: test -x ../sazed/docs/meshwork/meshwork && test -x ../leras/docs/meshwork/meshwork
docs:
  - .claude/skills/meshwork/references/install.md
discovered-from: mw-bds8yq5
status: open
created: 2026-08-10T18:34Z
seq: 260
blocked-reason:
needs: [mw-nx91erh, mw-rtt16df, mw-mcx59sd]
handoff: |
  Direction landed 2026-08-24: the reconciliation rides mw-nx91erh —
  next release ships a legacy-shim-to-plugin migration path, then adopter
  repos (sazed, leras first) install through the Claude Code plugin.
  Neither hook-bless nor a hand-installed shim proceeds on its own. This
  task stays as the install.md-reconcile marker, dep-blocked on
  mw-nx91erh. The current verify (grep SessionStart install.md || test -x
  ../sazed/meshwork) still detects one valid endpoint but re-shape it once
  migrate.md defines the modern layout. The 2026-08-12 evidence comment
  stays binding: MESHWORK_AUTHOR session tagging must survive migration or
  attribution silently falls to default_author.
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
- 2026-08-23T19:18Z open→blocked — awaiting owner direction pick: bless sazed's SessionStart hook pattern in install.md, or install the committed shim in sazed — evidence comment (2026-08-12) scores it shim-ward: zero --as in 35 shim-less sessions, three distinct $(cat) fragilities
- 2026-08-24T13:09Z blocked→open

## comments
- 2026-08-12T20:50Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] The shim-vs-hook divergence has a measured cost now. sazed ran the whole week shim-less: zero --as uses in 35 sessions, so all 8 agent comments and every claimed-by stamp read as the owner in prime's weather (fc237a1a, 4b5a9264). The $(cat .meshwork-version) incantation failed three distinct ways: wrong-cwd cat (63b829ba), a sandbox EPERM retried 4x that killed every meshwork verb for a session (632ce3d2), and a version-pinned settings.local.json allow-rule that rotted at the v0.2.0 upgrade (f6e7cfbc). A committed shim fixes attribution and all three fragilities at once; adopt.md should also warn against version-pinned permission rules.
- 2026-08-24T13:09Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Owner ruling 2026-08-24: neither option as posed. meshwork as a Claude plugin must migrate legacy shim deploys to modern plugin installs; the migration path ships in the next release, then all adopter repos upgrade and install through Claude Code plugins. sazed and leras pre-date the plugin and were set up manually by an agent. Filed mw-nx91erh to carry the release work; this task dep-blocks on it as the install.md-reconcile marker.
- 2026-08-24T13:32Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] migrate.md landed (mw-nx91erh closed 2026-08-24): the modern layout is defined — plugin skill, per-repo pin, committed docs/meshwork/meshwork shim carrying MESHWORK_AUTHOR. Re-shaped this verify accordingly: both adopter shims at the modern path (was: grep SessionStart install.md || sazed root shim). Dep-wired onto the per-repo sweeps mw-rtt16df (sazed) and mw-mcx59sd (leras), which need the release cut mw-h4s4gka. install.md itself was reconciled on mw-nx91erh — this task now closes when reality matches it.
