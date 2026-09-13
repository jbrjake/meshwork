---
id: cf-bund1ed
title: Bundled done task — the first document of a bundle
status: done
category: engine/spill
verify: exists README.md
created: 2026-07-31
---
A bundle (FORMAT.md §Bundles, format 2) is task documents concatenated:
this one opens the file. The bundle's name carries no id; each document's
`id:` line does. Readers load it exactly as a file of its own, with the
bundle as its path.

## log
- 2026-07-31 created
- 2026-08-05T09:00Z open→done — verify exit 0

---
id: cf-bund2ed
title: Bundled done task whose body quotes the format
status: done
verify: "true"
created: 2026-08-01
---
The boundary rule is `---` followed by an `id:` line, outside fences. Both
of these stay in this body:

```
---
id: cf-notadoc
title: fenced text, not a document
---
```

---

That bare rule above is a horizontal rule — no `id:` follows it.

## log
- 2026-08-01 created
- 2026-08-05T09:30Z open→done — verify exit 0

## comments
- 2026-08-06T10:00Z [spec] A comment appended into the bundle after compaction.
