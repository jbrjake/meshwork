---
id: mw-0ssk8dg
title: Project parent into the SQL tasks table, and make q errors name every table
status: open
category: core/query
verify: ./meshwork q "SELECT parent FROM tasks LIMIT 1"
relates:
  - mw-getx732
  - mw-jqj9qa9
created: 2026-08-12T20:48Z
---
`WHERE parent = …` is the first thing an agent guesses for
parent/child queries; today it dies with a Schema error that lists only
`tasks.*` columns — never mentioning `edges` exists — and a
`sqlite_master` probe dies too. The observed recovery (leras 4e5b1f04
13:44) was sampling `edges` blind and hand-building a 200-char
gid-prefixed JOIN, now hard-coded into four umbrella verifies.

Project `parent` onto `tasks` (it is one edge kind, child-points-up, so
the column is well-defined), and make the q error path enumerate the
queryable tables so the graph is discoverable without archaeology.

## log
- 2026-08-12T20:48Z created
