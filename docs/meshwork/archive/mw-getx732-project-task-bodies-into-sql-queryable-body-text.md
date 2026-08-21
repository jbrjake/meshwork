---
id: mw-getx732
title: Project task bodies into SQL (queryable body text)
status: done
category: core/format
verify: run cargo test e2e::body_projection
docs:
  - FORMAT.md#projection
  - DESIGN-meshwork.md#§-4-tables-the-sql-contract
created: 2026-08-09T23:11Z
seq: 220
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
- 2026-08-21T20:33Z open→doing — claimed by claude (session_016iEafFdzwyKAtsU3AEMhaU)
- 2026-08-21T20:37Z doing→done — verify exit 0 @ 603851b+3

## comments
- 2026-08-12T20:50Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Observed demand: sazed 4dc8792e tried coalesce(body,'') LIKE '%utf8%', got the Schema error, and fell back to grep -rlni over docs/meshwork/*.md. Title-only LIKE searches — the lossy substitute — appear in 8+ sazed sessions. Body projection is the missing half of the daily q driver.
- 2026-08-21T20:33Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Design settled before code: (1) a tasks.body COLUMN, not a seventh table — bodies already ride in memory on every load (Task.description); a table buys joins, no memory. (2) body = the description section (text before the first unfenced tail heading), trimmed; valid-but-empty projects '' while invalid rows project NULL — parsed-and-empty vs unknown stay distinguishable in SQL. (3) NO format bump: on-disk bytes are untouched and the projection widens additively — a bump would make every pinned binary in the portfolio refuse stores it still reads correctly. Column appended LAST so existing readers' column order is undisturbed. FORMAT.md projection + conformance corpus + DESIGN tables updated together.
