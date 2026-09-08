---
id: lf-bad0001
title: Nonconforming created stamp
status: open
category: engine
seq: 400
created: 2026-08-06T21:47-04:00
---
An offset stamp is nonconforming: `facts.created_at` is NULL, so this task
has no age, appears in no `flow` day and no `hazard` bin, and its
`events` row for `created` carries the stamp as written with `at` NULL.

## log
- 2026-08-06T21:47-04:00 created
