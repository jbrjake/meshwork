---
id: mw-xg67266
title: Rule on the §6 additions — stats, asks, portfolio search, show --comments N
category: meta/ruling
labels: [owner]
seq: 400
verify: contains docs/PLAN-proposals-implementation.md /R-D RULED 20[0-9]{2}-[0-9]{2}-[0-9]{2}/
docs:
  - docs/PLAN-proposals-implementation.md#§-3-rulings
  - docs/DESIGN-meshwork.md#§-6-cli-surface
  - docs/PROPOSAL-analytics.md#§-5-3-stats
status: done
created: 2026-09-07T16:32Z
---
Owner-gated. Four items. Each is a canned query or a flag over data the views already carry;
what is ruled is the surface, not the data. `e2e::cli_surface_frozen` is re-blessed once per
accepted verb with a reviewed diff.

## log
- 2026-09-07T16:32Z created
- 2026-09-21T00:06Z open→done — verify exit 0 @ 29da891+15
