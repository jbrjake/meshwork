---
id: mw-z0vh
title: "Release CI: cache cargo build artifacts — datafusion cold-builds ~1h per run"
status: done
category: meta/distribution
verify: grep -qi cache .github/workflows/release.yml
docs:
  - REQUIREMENTS-meshwork.md#§-j-non-functional   # MW-J3 adoptability
seq: 210
created: 2026-08-06
---

## log
- 2026-08-06 created
- 2026-08-06 open→done — verify exit 0
