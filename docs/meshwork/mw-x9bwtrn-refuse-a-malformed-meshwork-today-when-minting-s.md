---
id: mw-x9bwtrn
title: "Refuse a malformed MESHWORK_TODAY when minting stamps, as the view clock already does"
category: core/lifecycle
docs: [FORMAT.md#task-file]
verify: run cargo test mint_refuses_malformed_today
status: open
created: 2026-10-02T13:32Z
---
`MESHWORK_TODAY=banana meshwork add "x" --verify "exists y"` writes `created: banana` and a `- banana created` log line (reproduced in a scratch store). The same value makes `prime` and view queries refuse with `MESHWORK_TODAY must be YYYY-MM-DD or YYYY-MM-DDTHH:MMZ`. Minting (src/clock.rs) takes the override verbatim; it should apply the same check before anything is written.

## log
- 2026-10-02T13:32Z created
