---
id: mw-6tzr
title: "Flip this repo's own session ritual to self-host: CLAUDE.md step 1 -> meshwork prime; consider SessionStart hook"
status: done
category: meta/store
verify: grep -q 'meshwork prime' CLAUDE.md
docs:
  - DESIGN-meshwork.md#7-session-integration-where-the-savings-land
created: 2026-08-06
---
CLAUDE.md's session ritual still opens with "read the Position line in
PLAN-meshwork-build.md" — the pre-self-host ritual — so sessions grep docs
instead of running prime, even though the store went live in 1.8. Flip step 1
to `meshwork prime`; consider a SessionStart hook here (same wiring 1.9
installs in sazed). Observed in practice: 2026-08-06 session searched
PLAN/HANDOFF by hand.

## log
- 2026-08-06 created
- 2026-08-06 open→done — verify exit 0
