# HANDOFF (≤2KB — pointer, not narrative; durable state lives in PLAN Position + TRACE + commits)

**Done:** Through PLAN 1.2 (2026-08-04). `tree`: walks parent edges downward any depth; level names index by ABSOLUTE hierarchy depth (walk-up from the requested root), so `tree az-spr7` labels the same as the full tree — cosmetic only (MW-B8). `why`: DFS through unmet needs; frontier = blockers nothing further blocks; unresolved cross-repo/dangling refs surface as `{"ref","unresolved":true}` conservative entries (MW-G5). `blocked`: canned SQL, 20-cap + marker + `--all`. Goldens tree/why/blocked-alpha.json committed. TRACE: B8, C2 done (24 planned).

**Decisions:** tree output is uncapped (scoped by subtree — a detail view like show); why sorts frontier by id and dedups.

**Open threads:** none new.

**Next concrete step:** PLAN 1.3 — category segment-prefix + label queries (B4, B5): whole-segment prefix matching (`engine/spill` matches `engine/spill/compaction`, never `engine/spillover`).
verify: `cargo test query::category_labels` exits 0.
