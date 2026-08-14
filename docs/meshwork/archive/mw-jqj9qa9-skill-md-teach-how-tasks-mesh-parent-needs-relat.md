---
id: mw-jqj9qa9
title: "SKILL.md: teach how tasks mesh — parent/needs/relates, seq, ready deriving from the graph"
status: done
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
- 2026-08-14T12:47Z open→doing — claimed by claude (session_016iEafFdzwyKAtsU3AEMhaU)
- 2026-08-14T12:58Z doing→done — verify exit 0 @ c76478f+2

## comments
- 2026-08-12T20:50Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Hard usage data on what goes untaught: across 35 sazed sessions, dep was used zero times and tree/why/blocked near-zero — seq was the only lever agents reached for, while imported flag-marker titles contradicted the actual seq order. In leras, parent-progress queries had to be reverse-engineered live: q WHERE parent= hit the Schema error, then blind edges sampling, ending in a 200-char gid JOIN now hard-coded in four umbrella verifies (4e5b1f04). Teach the graph verbs and carry the JOIN idiom verbatim (mw-0ssk8dg tracks projecting parent so the idiom shrinks).
