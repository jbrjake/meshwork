---
id: mw-mrjccx2
title: "Registry durability: repo rename aliases + ID-alias collision lint"
status: open
category: core/portfolio
verify: cargo test e2e::registry_rename_alias
docs:
  - REQUIREMENTS-meshwork.md#§-g-portfolio
  - DESIGN-meshwork.md#§-9-portfolio-master-sequencing
seq: 18
created: 2026-08-06
---
Owner-accepted 2026-08-06 (format-hardening review). Cross-repo edges
bake repo#id into OTHER repos' files, and every task ID bakes the
store alias forever (config.toml says so) — but the registry has no
durability story for either. (a) repos.toml gains per-repo
`aliases = ["oldname"]` so inbound refs survive a repo rename;
resolution accepts old names, lint warns renamed-repo and suggests the
rewrite (never silent). (b) Nothing stops two repos claiming the same
ID alias prefix; portfolio lint gains an alias-collision error —
bare-ID lookup is ambiguous the moment it happens. Registry-format
decision, so it lands with/before 2.1 (mw-5ckb needs this).

## log
- 2026-08-06 created
