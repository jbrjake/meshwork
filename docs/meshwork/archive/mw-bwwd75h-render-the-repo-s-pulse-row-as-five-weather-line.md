---
id: mw-bwwd75h
title: Render the repo's pulse row as five weather lines in prime, computed in Rust and equal to the pulse view
category: product/prime
needs: [mw-zwgp6x7]
seq: 580
verify: run cargo test e2e::prime_pulse_matches_view
docs:
  - docs/PROPOSAL-analytics.md#§-5-2-prime
  - docs/PROPOSAL-analytics.md#§-4-11-pulse
status: done
created: 2026-09-07T16:32Z
---
Flow, queue, graph, asks, friction — each ≤ 160 bytes, the block ≤ 800, every count with its
denominator, zero-lines omitted; the budget rule (also-ready shrinks to 3, then friction, then
graph, each cut loud). The asks line rides the inbox's union read and is `mw-r6g9bhe`'s
headline; the queue line's triage count uses 14 days until `q-decay-wip` makes it config. The
next block gains `cites N closed tasks (ids…)` from a mention pass over the handoff. The
differential test asserts `SELECT * FROM pulse` equals the rendered numbers on every fixture
store; goldens re-blessed with a reviewed diff; `check-perf.sh` green. MW-S6/S7.

## log
- 2026-09-07T16:32Z created
- 2026-09-12T16:25Z handoff by claude (a4b15620-cf77-4910-ac26-1c233b1b0c38)
- 2026-09-12T16:25Z open→doing — claimed by claude (a4b15620-cf77-4910-ac26-1c233b1b0c38)
- 2026-09-12T16:40Z doing→done — verify exit 0 @ 6e25cea+15
