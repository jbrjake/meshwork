---
id: mw-e6qsjq0
title: Split registry.rs before the 750 ceiling
status: open
category: core/hygiene
verify: awk 'END{exit NR>500}' src/registry.rs
created: 2026-08-10T19:16Z
---

## log
- 2026-08-10T19:16Z created

## comments
- 2026-08-10T19:16Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Filed at 700/750 (mw-kkvs8zq's inbound scan landed there on top of mw-2nmsys2's sequence findings). Natural seam: loading/resolution (load, portfolio_dir, quiet_load, load_stores, resolve_foreign, load_sequence) vs registry-aware findings + scans (registry_findings and its helpers, sequence_findings, inbound_needs). Do it before the next portfolio feature touches the file — mw-908n9k2 and the mw-chcqk6g prune both will.
