# TRACE.md — requirement → test map (MW-J5)

Machine-checked by `verify_meshwork.sh` §6: every MW-* MUST must have a row; `done` rows must cite tests that exist (or be gate/pilot-satisfied); `planned` rows fail the gate under `--strict` (v1 acceptance). Flip a row to `done` only in the same commit as its passing test.

| req | tests | status |
|---|---|---|
| MW-A1 | `parse::roundtrip_hand_edited`, `e2e::add_show_roundtrip` | done |
| MW-A2 | `tables::memtable_no_disk`, `e2e::cache_delete_safe` | done |
| MW-A3 | `e2e::init_layout` (nothing written outside repo, no hooks) | done |
| MW-A4 | `id::collision_reroll`, `e2e::merge_duplicate_id` | done |
| MW-A5 | `lint::description_size_warn`, `e2e::show_caps` | done |
| MW-A6 | `parse::unknown_field_warns` | done |
| MW-B1 | `tables::edge_kinds`, `e2e::dep_edit` | done |
| MW-B2 | `lint::cycle_needs`, `lint::cycle_parent` | done |
| MW-B3 | `e2e::crossrepo_resolution`, `lint::parent_crossrepo_error` | planned |
| MW-B4 | `query::category_segment_prefix` | done |
| MW-B5 | `query::labels_orthogonal` | done |
| MW-B6 | `e2e::ready_golden` (container-exclusion + unresolved-blocks cases) | done |
| MW-B7 | `lint::parent_rollup_warn` | done |
| MW-B8 | `e2e::tree_why_blocked_golden` (5-deep chain, cosmetic level names) | done |
| MW-C1 | `e2e::raw_sql_tables` | done |
| MW-C2 | `e2e::tree_why_blocked_golden` | done |
| MW-C3 | `e2e::json_stable_shapes` | done |
| MW-C4 | gate §7 (`perf::ready_1k`, `perf::portfolio_20x`) | planned |
| MW-D1 | `e2e::show_caps`, `e2e::ready_golden` | done |
| MW-D2 | `e2e::caps_and_more_marker` | done |
| MW-D3 | `e2e::prime_budget` | done |
| MW-D4 | `e2e::cli_surface_frozen` | done |
| MW-D5 | `unit::budget_bytes_not_lines` | done |
| MW-E1 | `e2e::transitions` | done |
| MW-E2 | `e2e::close_gating`, `e2e::close_waive_recorded` | done |
| MW-E3 | `e2e::log_append_on_transitions`, `e2e::log_table_minted_forms` (mw-3wnhhvp: the record made queryable) | done |
| MW-E4 | `e2e::discovered_from_edge` | done |
| MW-E5 | `e2e::verify_trust_gate_refuses_unapproved`, `e2e::verify_trust_changed_text_revokes`, `e2e::verify_trust_env_grant_for_ci` | done |
| MW-F1 | `parse::docs_links` | done |
| MW-F2 | `e2e::show_docs_excerpts` | planned |
| MW-F3 | `lint::anchor_missing_warn` | planned |
| MW-K1 | `e2e::comment_attach`, `parse::comment_format` | done |
| MW-K2 | `e2e::comment_attach` | done |
| MW-K3 | `lint::attachment_size_warn` | done |
| MW-K4 | `e2e::add_show_roundtrip` | done |
| MW-G1 | `e2e::repo_self_contained` | planned |
| MW-G2 | `e2e::registry_overrides` | planned |
| MW-G3 | `e2e::portfolio_union_golden` | planned |
| MW-G4 | `e2e::portfolio_next_ordering` | planned |
| MW-G5 | `e2e::absent_repo` | planned |
| MW-H1 | `e2e::mirror_create`, `e2e::mirror_append` | planned |
| MW-H2 | `e2e::mirror_never_mutates` | planned |
| MW-H3 | `e2e::mirror_idempotent` | planned |
| MW-H4 | `e2e::mirror_status_reports_only` | planned |
| MW-H5 | `e2e::offline_all` | done |
| MW-I1 | `e2e::merge_concurrent_worktrees` | done |
| MW-I2 | `e2e::merge_union_poison`, `e2e::invalid_visible` | done |
| MW-J1 | gate §§1–2 + Cargo.toml dep review (done at B0) | planned |
| MW-J2 | gate §§4–5 | planned |
| MW-J3 | `e2e::import_todo_golden` + M1.9 pilot checklist | planned |
| MW-J4 | `fixtures::corpus_covers_features` | done |
| MW-J5 | gate §6 (this file's own checker) | planned |
| MW-J6 | `e2e::offline_all` + stub-gh harness | done |
