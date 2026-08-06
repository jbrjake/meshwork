---
id: mw-0pj8qgv
title: Ship the v0.1.4 release
status: blocked
category: meta/distribution
verify: gh release view v0.1.4 -R jbrjake/meshwork --json assets --jq '.assets | length' | grep -q 2
seq: 9
created: 2026-08-06
blocked-reason: "Actions outage AND structural issue found 2026-08-06: v0.1.4 tag (74d65e2) predates the workflow_dispatch commit (040768a), so dispatch --ref v0.1.4 422s permanently — that ref's workflow has no dispatch trigger. Unblock when webhooks recover: delete + re-push the tag (same commit; its workflow builds darwin+skill = 2 assets, matching verify). Dispatch re-fires only work for tags cut after 040768a."
---

## log
- 2026-08-06 created
- 2026-08-06 open→blocked — GitHub Actions major outage — webhook triggers throttled, tag pushes create no runs, dispatch API refused (definition indexing rides the same pipeline). Unblock: Actions recovers, then gh workflow run release --ref v0.1.4 (or delete+re-push the tag), verify both assets + binary reports 0.1.4.
- 2026-08-06 blocked→open
- 2026-08-06 open→blocked — Actions outage AND structural issue found 2026-08-06: v0.1.4 tag (74d65e2) predates the workflow_dispatch commit (040768a), so dispatch --ref v0.1.4 422s permanently — that ref's workflow has no dispatch trigger. Unblock when webhooks recover: delete + re-push the tag (same commit; its workflow builds darwin+skill = 2 assets, matching verify). Dispatch re-fires only work for tags cut after 040768a.

## comments
- 2026-08-06 [claude] 22:30 UTC re-check: githubstatus still major outage; deployed fix has new-workflow success at 97% but webhook triggers still throttled and dispatch API still 422s ('Workflow does not have workflow_dispatch trigger' — definition re-index hasn't happened). Left blocked; will re-try later in session.
