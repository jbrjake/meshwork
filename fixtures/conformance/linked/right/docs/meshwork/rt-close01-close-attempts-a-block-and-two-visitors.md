---
id: rt-close01
title: Two failed closes, one block, two identities since it last moved
status: open
category: engine
verify: exists docs/meshwork/config.toml
seq: 30
created: 2026-08-04T08:00Z
---
`facts.close_attempts` 2, `blocks` 1; the two comments after the last
transition come from two identities, so `attention.touched_since_move`
is 2 and `pulse.thrash_n` counts this task for `right`.

## log
- 2026-08-04T08:00Z created
- 2026-08-05T09:00Z open→blocked — waiting on rt-dep0001
- 2026-08-06T09:00Z blocked→open — unblocked by hand
- 2026-08-07T09:00Z close attempt — verify exit 1
- 2026-08-07T09:30Z close attempt — verify exit 1

## comments
- 2026-08-08T10:00Z [claude (session_right)] Verify still red; the fixture path moved.
- 2026-08-09T10:00Z [Jon Rubin] Leave it; the path is right, the verify is wrong.
