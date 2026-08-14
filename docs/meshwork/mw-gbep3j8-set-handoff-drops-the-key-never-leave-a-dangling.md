---
id: mw-gbep3j8
title: set --handoff '' drops the key — never leave a dangling empty block scalar
status: open
category: core/authoring
discovered-from: mw-5zn3ern
verify: out=$(cargo test set_handoff_clear 2>&1) && echo "$out" | grep -qE "ok\. [1-9][0-9]* passed"
created: 2026-08-14T14:53Z
---

## log
- 2026-08-14T14:53Z created

## comments
- 2026-08-14T14:53Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Observed while clearing 9 stale handoffs (mw-5zn3ern): set --handoff '' rewrote each frontmatter with a dangling 'handoff: |' line — an empty block scalar. Lint treats it as absent (no handoff-stale warn), so the state is legal but vestigial; the 9 files were hand-cleaned via batch_edit. Fix in the set verb's frontmatter writer: empty string clears the key entirely. Red-checked verify 2026-08-14 (exit 1).
