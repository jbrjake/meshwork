---
id: mw-5xdyxep
title: "Add full-text search across task content — bodies, handoffs, and archives are unsearchable today"
category: core/query
labels: [feature]
verify: run cargo test full_text_search
relates:
  - mw-getx732
seq: 50
status: open
created: 2026-08-21T19:32Z
---
Owner ask (2026-08-21): far too much store content is unsearchable. The
surface delta this implies (DESIGN §6 is frozen) is owner-initiated, not
scope creep — the design session should record the ruling alongside the
implementation plan.

What search reaches today: `q` sees `title`, `comments.text`, and
`log.note`. It does NOT see the free-markdown body (the bulk of most
tasks) or `handoff:` blocks; the only recourse is `grep -r
docs/meshwork/`, which cannot join against status, edges, or category,
and which agents without repo context never think to run.

Design questions to settle before code:

- Surface: a `search <term>` verb, or sugar over `q` once body text is
  projected? [[mw-getx732]] (body projection into SQL) is the natural
  substrate — decide whether this task `needs:` it or subsumes it.
- Corpus: title + body + comments + log + handoff; archived tasks are
  already loaded and must be included — stale knowledge lives there.
- Semantics: substring/ILIKE is probably enough at store scale (perf
  budgets: ready+prime <100ms @1K tasks) — real FTS ranking only if
  cheap. DataFusion offers LIKE/regexp; there is no index to maintain.
- Output: task hits with matched-field context, honoring the listing cap
  and the JSON envelope contract.

## log
- 2026-08-21T19:32Z created
