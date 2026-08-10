---
id: mw-jqj9qa9
title: "SKILL.md: teach how tasks mesh — parent/needs/relates, seq, ready deriving from the graph"
status: open
category: skill
discovered-from: mw-9zrd
verify: grep -q -- '--parent' .claude/skills/meshwork/SKILL.md && grep -q 'relates' .claude/skills/meshwork/SKILL.md
seq: 80
docs:
  - DESIGN-meshwork.md#§-7-session-integration
  - REQUIREMENTS-meshwork.md#§-b-graph-model
created: 2026-08-10T22:22Z
---
leras evidence (2026-08-10 migration review): the import-review pass produced
titles, verifies, and seq only — every parent/needs/relates edge was
retrofitted after an owner mid-session nudge. SKILL.md's Rules name seq,
verify:, docs: but never the edge fields, nor the queue mechanics that make
them matter: ready hides a parent while children live, needs gates
visibility, priority is edges+seq — never list order. Add a short "How tasks
mesh" block to the Rules: parent for section umbrellas, needs for hard
order, relates for coupling, and the ready-derivation sentence. Daily-use
terse — mechanics live here, migration ritual stays in adopt.md.

## log
- 2026-08-10T22:22Z created
