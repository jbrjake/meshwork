---
id: mw-0pj8qgv
title: Ship the v0.1.4 release
status: blocked
category: meta/distribution
verify: gh release view v0.1.4 -R jbrjake/meshwork --json assets --jq '.assets | length' | grep -q 2
seq: 9
created: 2026-08-06
blocked-reason: "GitHub Actions major outage — webhook triggers throttled, tag pushes create no runs, dispatch API refused (definition indexing rides the same pipeline). Unblock: Actions recovers, then gh workflow run release --ref v0.1.4 (or delete+re-push the tag), verify both assets + binary reports 0.1.4."
---

## log
- 2026-08-06 created
- 2026-08-06 open→blocked — GitHub Actions major outage — webhook triggers throttled, tag pushes create no runs, dispatch API refused (definition indexing rides the same pipeline). Unblock: Actions recovers, then gh workflow run release --ref v0.1.4 (or delete+re-push the tag), verify both assets + binary reports 0.1.4.
