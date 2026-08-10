---
id: mw-a6jdf5s
title: "Constrain alias to [a-z0-9]+ — dashed aliases corrupt ID recovery"
category: core/format
verify: cargo test format::alias_charset
docs:
  - FORMAT.md#configtoml
  - FORMAT.md#task-file
status: done
created: 2026-08-09T23:17Z
seq: 30
---
Review finding (2026-08-09). ID recovery from an invalid file takes
"the first two dash-segments of the stem," and config.toml says `alias`
is just "string, required." An alias like `my-repo` silently corrupts
recovery (`my-repo-abc1234-slug.md` recovers as `my-repo`). Constrain
it to `[a-z0-9]+` in the config table — one line, kills a whole class.
Enforce at `init`/`lint`; existing single-segment aliases are untouched.

## log
- 2026-08-09T23:17Z created
- 2026-08-10T13:59Z open→doing — claimed by claude (session_016iEafFdzwyKAtsU3AEMhaU)
- 2026-08-10T14:02Z doing→done — verify exit 0 @ 3634eee+7
