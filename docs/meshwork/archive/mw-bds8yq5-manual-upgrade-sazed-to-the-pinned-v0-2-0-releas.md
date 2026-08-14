---
id: mw-bds8yq5
title: "Manual: upgrade sazed to the pinned v0.2.0 release (binary + skill)"
category: meta/distribution
seq: 30
verify: grep -qx v0.2.0 ../sazed/.meshwork-version
docs:
  - .claude/skills/meshwork/references/install.md
status: done
created: 2026-08-10T16:31Z
---
sazed pins v0.1.5; v0.2.0 published 2026-08-10 (tag at 775a7fd, 4 platform
binaries + skill tarball). Per-repo ritual, nothing global: bump
.meshwork-version, install the pinned binary under ~/.meshwork/versions/,
refresh the repo's skill copy from the release tarball, then prime + lint
on sazed's own store under the new pin. Lands before the leras migration
so both live stores run the same engine when the portfolio unions them.

## log
- 2026-08-10T16:31Z created
- 2026-08-10T18:32Z open→done — verify exit 0 @ f831613+1

## comments
- 2026-08-10T18:32Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Done in sazed: 1f3a16e (pin bump + binary + skill refresh from the v0.2.0 release; skill now ships references/adopt.md). prime + lint exit 0 under the new engine — the one lint error was a pre-existing store defect (sa-va0tvyx unquoted verify: with a colon, invisible to ready/prime), fixed separately as 7693a05; v0.1.5 reproduced the identical error, so v0.2.0 adds no new failures. Note: sazed has no committed ./meshwork shim — it resolves the pin via a SessionStart hook instead of the install.md shim ritual.
