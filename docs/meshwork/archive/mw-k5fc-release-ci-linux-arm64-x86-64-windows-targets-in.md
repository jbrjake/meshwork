---
id: mw-k5fc
title: "Release CI: linux arm64/x86_64 + windows targets in the matrix"
status: done
category: meta/distribution
verify: grep -q linux .github/workflows/release.yml && grep -qi windows .github/workflows/release.yml
docs:
  - REQUIREMENTS-meshwork.md#§-j-non-functional   # MW-J3 adoptability
seq: 200
created: 2026-08-06
---

## log
- 2026-08-06 created
- 2026-08-06 open→done — verify exit 0
