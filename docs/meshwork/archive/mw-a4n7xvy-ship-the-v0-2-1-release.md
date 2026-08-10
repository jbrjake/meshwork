---
id: mw-a4n7xvy
title: Ship the v0.2.1 release
status: done
category: meta/distribution
verify: gh release view v0.2.1 -R jbrjake/meshwork --json assets --jq '.assets | length' | grep -qx 5
created: 2026-08-10T19:41Z
---

## log
- 2026-08-10T19:41Z created
- 2026-08-10T19:41Z open→doing — claimed by claude (session_016iEafFdzwyKAtsU3AEMhaU)
- 2026-08-10T19:51Z doing→done — verify exit 0 @ b82b4f4
