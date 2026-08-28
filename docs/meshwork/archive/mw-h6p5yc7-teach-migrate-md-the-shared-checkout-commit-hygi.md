---
id: mw-h6p5yc7
title: Teach migrate.md the shared-checkout commit hygiene
status: done
category: meta/distribution
discovered-from: mw-rtt16df
verify: contains .claude/skills/meshwork/references/migrate.md pathspec
docs:
  - .claude/skills/meshwork/references/migrate.md
seq: 250
created: 2026-08-28T14:53Z
---

The self-migration audit (owner ask 2026-08-28) found exactly one
instruction living in the sazed/leras migration tasks but not in the
references: adopter repos often have live agent sessions sharing the
checkout, so a migrating session must check what is already staged and
commit by explicit pathspec. Fold that line into migrate.md step 8 so a
repo migrating itself from the plugin alone has the complete ritual.
Ships with the next release; the 0.4.0 plugin copy stays as cut.

## log
- 2026-08-28T14:53Z created
- 2026-08-28T14:54Z open→doing — claimed by claude (session_016iEafFdzwyKAtsU3AEMhaU)
- 2026-08-28T14:54Z doing→done — verify exit 0 @ 4b4c6d2+4
