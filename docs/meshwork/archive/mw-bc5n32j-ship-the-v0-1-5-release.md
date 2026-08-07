---
id: mw-bc5n32j
title: Ship the v0.1.5 release
status: done
category: meta/distribution
verify: gh release view v0.1.5 -R jbrjake/meshwork --json assets --jq '.assets | length' | grep -qx 5
seq: 9
created: 2026-08-07T03:31Z
---

## log
- 2026-08-07T03:31Z created
- 2026-08-07T03:47Z open→done — verify exit 0 @ 6ba0952
