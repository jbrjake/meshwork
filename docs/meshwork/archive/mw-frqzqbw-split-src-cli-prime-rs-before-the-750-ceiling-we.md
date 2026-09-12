---
id: mw-frqzqbw
title: "Split src/cli/prime.rs before the 750 ceiling — weather, next block and assembly into a render module"
status: done
category: core/hygiene
verify: "all(contains src/cli/prime_render.rs weather_lines, run cargo test prime_handoff_sections)"
created: 2026-09-12T16:46Z
---

prime.rs stands at 725 lines after the pulse block, the inbox pointer and the footer landed; the ceiling is 750 and the target 500. Move the renderers (weather_lines, handoff_tag, next_block_lines, also_ready_lines, recent_dones, advisory_lines) and the Digest/assemble pair into src/cli/prime_render.rs, leaving run() and the JSON emitter behind. Pure motion: every prime test stays green untouched; check-perf's prime_1k baseline holds.

## log
- 2026-09-12T16:46Z created
- 2026-09-12T17:14Z open→doing — claimed by claude (a4b15620-cf77-4910-ac26-1c233b1b0c38)
- 2026-09-12T17:18Z doing→done — verify exit 0 @ 08c7ee2+4
