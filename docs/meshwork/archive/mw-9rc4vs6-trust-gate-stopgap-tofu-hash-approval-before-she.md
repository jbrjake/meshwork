---
id: mw-9rc4vs6
title: "Trust gate stopgap: TOFU hash approval before shell verify runs"
status: done
category: core/verify
needs: [mw-mjwfvxn]
parent: mw-6895bkg
verify: cargo test e2e::verify_trust
seq: 16
docs:
  - REQUIREMENTS-meshwork.md#§-e-lifecycle-discipline # MW-E2 sh -c today
  - DESIGN-meshwork.md#§-6-cli-surface # frozen verb table
created: 2026-08-06
---
The direnv-allow pattern, minimal: SHA-256 over (id, verify text)
recorded in meshwork/.cache/ (already gitignored — approvals are
per-clone, never merge in from a PR). close refuses an unapproved or
changed shell verify, loudly, and names the approval step; approval is
explicit and requires the text on screen (show first). Test/CI hook per
mw-mjwfvxn ruling. No sandbox, no parsing, ~a day of code — this alone
closes the drive-by PR → close → RCE path portfolio-wide, which is why
it jumps the queue (seq 16) while the DSL waits behind v1.

## log
- 2026-08-06 created
- 2026-08-07T02:28Z open→doing — claimed by claude
- 2026-08-07T02:35Z doing→done — verify exit 0

## comments
- 2026-08-07T02:31Z [claude] Landed: src/trust.rs (model layer, arch-guarded) — SHA-256 over id NUL verify-text, one hex line per approval in docs/meshwork/.cache/trusted-verifies (self-gitignored dir; deletion just re-gates, MW-A2). close refuses unapproved shell verifies pre-exec with the text on screen and the approval step named; close --approve prints + records + runs; MESHWORK_TRUST=1 honored per the mw-mjwfvxn ruling. Test suite grants MESHWORK_TRUST=1 in the shared helper (the suite checkout IS reviewed); trust tests strip it. sha2 declared in Cargo.toml (was already in-tree via datafusion). TRACE MW-E5 → done.
