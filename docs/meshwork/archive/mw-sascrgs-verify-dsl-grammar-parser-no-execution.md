---
id: mw-sascrgs
title: "Verify DSL: grammar + parser (no execution)"
status: done
category: core/verify
needs: [mw-mjwfvxn]
parent: mw-6895bkg
verify: out=$(cargo test verify_dsl::grammar 2>&1) && echo "$out" | grep -qE 'ok\. [1-9][0-9]* passed'
seq: 160
docs:
  - REQUIREMENTS-meshwork.md#§-e-lifecycle-discipline # MW-E2 sh -c today
  - REQUIREMENTS-meshwork.md#§-3-non-goals # ruling recorded here when scope moves
created: 2026-08-06
---
Small declarative predicate grammar covering observed verify shapes
(this store today: cargo test <filter>, grep -q <lit> <file>, test -f):
exists <path> / absent <path> / contains <path> <lit|/regex/> /
run <runner> <args> / all(...). Parse, don't eval — no shell, so
metacharacters have no meaning. Per-runner argv grammars, not an
argv[0] allowlist: args match tight character classes, no leading dash
(kills flag smuggling à la Cursor GHSA-hf2x-r83r-qw5q / Flowise).
Anything that doesn't parse as DSL is legacy shell → trust-gate path
(mw-9rc4vs6); the verify: key stays one string. Parser + golden corpus
only — no execution in this item.

## log
- 2026-08-06 created
- 2026-08-14T15:11Z open→doing — claimed by claude (session_016iEafFdzwyKAtsU3AEMhaU)
- 2026-08-14T15:16Z doing→done — verify exit 0 @ e27e13c+9
