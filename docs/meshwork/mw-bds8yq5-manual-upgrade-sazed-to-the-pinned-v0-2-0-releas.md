---
id: mw-bds8yq5
title: "Manual: upgrade sazed to the pinned v0.2.0 release (binary + skill)"
category: meta/distribution
seq: 30
verify: grep -qx v0.2.0 ../sazed/.meshwork-version
docs:
  - .claude/skills/meshwork/references/install.md
status: open
created: 2026-08-10T16:31Z
handoff: |
  v0.2.0 is live (tag 775a7fd). sazed pins v0.1.5 — bump
  .meshwork-version, install per the skill's references/install.md,
  refresh sazed's skill copy from the release tarball, then prime + lint
  there on the new pin. Repo-local everything; nothing global.
---
sazed pins v0.1.5; v0.2.0 published 2026-08-10 (tag at 775a7fd, 4 platform
binaries + skill tarball). Per-repo ritual, nothing global: bump
.meshwork-version, install the pinned binary under ~/.meshwork/versions/,
refresh the repo's skill copy from the release tarball, then prime + lint
on sazed's own store under the new pin. Lands before the leras migration
so both live stores run the same engine when the portfolio unions them.

## log
- 2026-08-10T16:31Z created
