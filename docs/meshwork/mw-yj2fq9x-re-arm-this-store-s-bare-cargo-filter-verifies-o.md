---
id: mw-yj2fq9x
title: "Re-arm this store's bare cargo-filter verifies — observed passes, not exit 0"
status: open
category: meta/store-hygiene
discovered-from: mw-9zrd
relates: [mw-8kfqz2z, mw-4aqmf0t, mw-0y66mhb]
verify: bad=0; for f in docs/meshwork/mw-*.md; do grep -qE '^status:[[:space:]](open|blocked)' "$f" || continue; grep -m1 '^verify:' "$f" | grep -qE '^verify:[[:space:]]"?cargo test [^|&;]*$' && bad=1; done; test $bad -eq 0
seq: 75
docs:
  - REQUIREMENTS-meshwork.md#§-e-lifecycle-discipline
created: 2026-08-10T22:22Z
---
Sweep observed 2026-08-10, this session: 39 of 48 open-task verifies exit 0
today. Every bare cargo-test filter names a future test, and a zero-match
filter still exits 0 — close would pass with the work undone across most of
the store, including mw-175bn4c itself (the red-check task). Also
mw-0y66mhb's verify pipes smoke through grep -v, reporting grep's exit —
green even when smoke fails. Re-arm each verify to the observed-pass idiom
(see @traps) without changing which test it names; give the piped one
smoke's own exit. Rewriting verify lines trips the approval ledger — expect
verify-changed-since-approval findings and a re-bless (mw-hz1ezcg,
mw-yyf1bab). Cheap textual pass now; the DSL migration (mw-4aqmf0t) will
rewrite them again later — don't wait for it.

## log
- 2026-08-10T22:22Z created
