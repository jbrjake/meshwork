---
id: mw-getx732
title: Project task bodies into SQL (queryable body text)
status: open
category: core/format
verify: out=$(cargo test e2e::body_projection 2>&1) && echo "$out" | grep -qE 'ok\. [1-9][0-9]* passed'
docs:
  - FORMAT.md#projection
  - DESIGN-meshwork.md#§-4-tables-the-sql-contract
created: 2026-08-09T23:11Z
---

The free-markdown description is the only task text absent from the SQL
projection — `tasks` carries structured fields, and searchable text stops at
`title`, `comments.text`, and `log.note`. Body search today falls back to
`grep -r docs/meshwork/`, which can't join against status/edges and is
invisible to `q`/`portfolio q`.

Proposal: project the body — either a `tasks.body` column or a seventh table.
Design-time questions to settle before code:

- FORMAT.md's projection is a versioned contract third-party readers implement
  from: adding a column means a format version bump and a normative extraction
  spec (the span between the frontmatter close fence and the first tail
  section, byte-exact — no normalization, so determinism holds; define empty
  body as `''` vs NULL).
- Not a query DSL (REQUIREMENTS §3 fence intact): SQL stays the only query
  surface; this adds no verb and no flag, it widens the existing tables.
- Cost: bodies ride into every load — check memory and the engine constant at
  1K tasks against mw-xjyhs9y's bench before committing to a column vs a
  lazily-registered table.

Origin: session question "what's the syntax to query the task body in SQL?"
(2026-08-09) — answer today is "you can't".

## log
- 2026-08-09T23:11Z created

## comments
- 2026-08-12T20:50Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Observed demand: sazed 4dc8792e tried coalesce(body,'') LIKE '%utf8%', got the Schema error, and fell back to grep -rlni over docs/meshwork/*.md. Title-only LIKE searches — the lossy substitute — appear in 8+ sazed sessions. Body projection is the missing half of the daily q driver.
