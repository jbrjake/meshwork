---
id: mw-a8tv
title: "prime grows handoff sections: next-task detail + last-N done; retire hand-written HANDOFF.md"
status: done
category: product/prime
verify: cargo test e2e::prime_handoff_sections
docs:
  - DESIGN-meshwork.md#§-7b-prime-as-materialized-handoff   # THE spec, owner-ruled
  - REQUIREMENTS-meshwork.md#§-d-context-discipline   # MW-D3, MW-D5
created: 2026-08-06
---
Owner-ruled over four design rounds (2026-08-06); full spec is DESIGN §7b —
prime becomes the materialized handoff (top-5 category rollup by min seq,
derived weather, next-task block led by its `handoff:` commentary, also-
ready, recent dones) and new frontmatter key `handoff:` carries the only
authored piece, on up-next tasks only, no verb, §6 untouched. Hand-written
docs/HANDOFF.md dies in the landing commit (see §7b for the full teardown
list). Build red-first from e2e::prime_handoff_sections; expect one
deliberate --bless of the prime golden; bodies truncate cleanly, never
overflow the 6KB cap. Blocks mw-ntt5 — the pilot installs the post-HANDOFF
world.

## log
- 2026-08-06 created
- 2026-08-06 open→done — verify exit 0
