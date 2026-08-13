---
id: mw-yj2fq9x
title: "Re-arm this store's bare cargo-filter verifies — observed passes, not exit 0"
status: done
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
- 2026-08-13T23:24Z open→doing — claimed by claude (session_016iEafFdzwyKAtsU3AEMhaU)
- 2026-08-13T23:32Z doing→done — verify exit 0 @ 274b51d+40

## comments
- 2026-08-13T23:32Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Swept 2026-08-13: 38 verifies re-armed via batch_edit spec — 37 bare cargo filters wrapped in the observed-pass idiom (out=$(cargo test F 2>&1) && echo "$out" | grep -qE 'ok\. [1-9][0-9]* passed'; mw-6895bkg's trailing-:: filter took the YAML double-quoted form), plus mw-0y66mhb's piped smoke recast to smoke's own exit AND the parse.rs-gone intent. No filter changed which test it names. Observed through sh -c: green path exit 0 on a real test, future-test path exit 1 (not 127) on mw-cvw8's mirror_create. Store lints 0 errors; approval ledger re-armed by construction — closes will prompt --approve once per verify.
