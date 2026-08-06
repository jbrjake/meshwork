---
id: bz-r34d
title: Retry policy for fetch
status: open
category: reader
labels: [reliability]
verify: "cargo test -p beta-reader retry::"
created: 2026-08-01
---
Bounded exponential backoff; give up loudly after 3.
