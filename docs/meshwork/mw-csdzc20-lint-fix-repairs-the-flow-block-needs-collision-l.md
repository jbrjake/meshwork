---
id: mw-csdzc20
title: "lint --fix repairs the flow+block `needs:` collision left by pre-fix `dep add`"
category: core/hygiene
labels: [bug]
verify: run cargo test lint_fix_needs_collision
discovered-from: mw-nzeezr8
seq: 30
status: open
created: 2026-08-21T19:08Z
---
Binaries before the mw-nzeezr8 fix rewrote a block-style `needs:` in flow
style while leaving the old indented items stranded beneath it:

```yaml
needs: [wo-a, wo-b]
  - wo-a
```

The file is invalid YAML, so it drops out of the tables entirely; `lint`
can only count it as invalid. The writing path is fixed, but stores
written by older pinned binaries can still carry the damage, and today
`lint --fix` cannot repair it.

The signature is mechanical and safe to repair at the text level: a
flow-style `needs: [...]` line immediately followed by indented `- item`
lines whose items are a subset of the flow list. The flow line is the
newer truth — `lint --fix` drops the stray indented lines, revalidates,
and reports the repair like the other mechanical-damage classes.

## log
- 2026-08-21T19:08Z created
