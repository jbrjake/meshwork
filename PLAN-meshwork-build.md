# PLAN-meshwork-build.md

**Status: BUILD NOT AUTHORIZED — this document is the go-button.** On "go", execute §1–§2 top-to-bottom; every decision is already made (DESIGN §15), every work item carries a `verify:` command (CLAUDE-BASELINE rule), and the gate (DESIGN §14) is built before the first feature. Companion to REQUIREMENTS-meshwork.md (`MW-*`) and DESIGN-meshwork.md (`§*`). 2026-08-04.

Conventions: each item is done only when its `verify:` exits 0 AND `./verify_meshwork.sh` stays green. Items are ordered; no item starts before its predecessors' verifies pass. House numbers apply to meshwork's own code: 500 warn / 750 fail per file, 80% coverage, N≥7 bench reps.

**Position: next = 0.8.** (Through 0.7 done 2026-08-04. `close`: verify via `sh -c` from repo root (subdir-proof), exit+date recorded in log on success AND failure, closes on 0 only; no-verify demands `--waive`; waive writes the queryable `waived` column. TRACE: E1, E2, E3 done.)

## 1. Bootstrap (B0–B4, first session)

| id | item | verify |
|---|---|---|
| B0 ✓ | crate at `~/Documents/code/meshwork` (repo pre-created by owner); deps pinned: clap 4, serde 1, `serde_yaml_ng`, serde_json, toml, datafusion 51 (sahjhan's major), tokio (rt), thiserror 2; dev: assert_cmd, predicates, tempfile | `cargo build` ✓ 2026-08-04 |
| B1 ✓ | `verify_meshwork.sh` — all 8 sections (DESIGN §14); §§3–4 pass trivially on the empty crate; §6 runs in *pending* mode (TRACE.md rows may be marked `planned`; the v1 acceptance run uses `--strict`, where `planned` fails) | `./verify_meshwork.sh` ✓ 2026-08-04 |
| B2 ✓ | Commit `TRACE.md` seeded from §3 below, all rows `planned` | `./verify_meshwork.sh` (§6 pending mode) ✓ 2026-08-04 |
| B3 ✓ | Fixture corpus skeleton: `fixtures/{alpha,alpha-broken,beta,portfolio,golden}` per DESIGN §13, plus `fixtures::corpus_covers_features` — a test asserting the corpus contains ≥1 instance of every feature/failure DESIGN §13 lists (this test keeps the corpus honest forever) | `cargo test fixtures::` ✓ 2026-08-04 |
| B4 ✓ | Stub `gh`: `tests/bin/gh` records argv+stdin to a `.calls` file, replays canned JSON from `tests/canned/`; harness prepends it to `$PATH` | `cargo test stub_gh::` ✓ 2026-08-04 |

## 2. Work items by milestone

### M0 — store, parse, core verbs (stop-line: usable in sazed pilot)

| id | item (MW refs) | verify |
|---|---|---|
| 0.1 ✓ | Task-file parser: strict serde model, frontmatter + `## log`/`## comments` tail sections, bullet+continuation comment format; unknown keys warn; parse failure → `invalid` row carrying filename-recovered ID (A1, A6, I2, K1) | `cargo test parse::` ✓ 2026-08-04 |
| 0.2 ✓ | ID generation: `<alias>-<4-char base32>`, collision re-roll against local files; seedable RNG hook for tests (A4) | `cargo test id::` ✓ 2026-08-04 |
| 0.3 ✓ | Ingestion → Arrow `MemTable`s → DataFusion `SessionContext`, five tables incl. `waived`, `ord`, `resolved`, child→parent edge direction (§3–4; C1) | `cargo test tables::` ✓ 2026-08-04 |
| 0.4 ✓ | `init`: writes `meshwork/` layout, config.toml, `.gitattributes` (`tasks/*.md merge=union`), `.cache/.gitignore`; refuses outside a git repo (A3, I1) | `cargo test e2e::init_layout` ✓ 2026-08-04 |
| 0.5 ✓ | `add` (all flags incl. `--verify`) + `show` (last-3 comments, `… and N more`) (A5, D2, E4, K4) | `cargo test e2e::add_show_roundtrip` ✓ 2026-08-04 |
| 0.6 ✓ | Transitions `start/block/drop/reopen` + log append; `block` demands `--reason` (E1, E3) | `cargo test e2e::transitions` ✓ 2026-08-04 |
| 0.7 ✓ | `close`: runs `verify:` via `sh -c` from repo root, records exit+date, closes on 0 only; `--waive` writes `waived` (E2) | `cargo test e2e::close_gating` ✓ 2026-08-04 |
| 0.8 | `ready` (normative SQL: needs clause + container clause) + `q` + `--json` everywhere, stable versioned shape (B6, C1, C3, D1, D2) | `cargo test e2e::ready_golden e2e::json_stable_shapes` |
| 0.9 | `lint`: schema, needs/parent cycles, cross-repo parent, blocked-without-reason, duplicate IDs, duplicate frontmatter keys, dangling edges, missing verify (warn); `--fix`: re-slug fewer-inbound side + rewrite same-repo refs, repair duplicate keys (A4, A6, B2, B3, I2) | `cargo test e2e::lint_broken_corpus` |
| 0.10 | Merge scenarios 1–3 (concurrent-worktrees, duplicate-id-merge, union-poison) (I1, I2, A4) | `cargo test e2e::merge_` |
| 0.11 | Offline-everything scenario 5 (H5) | `cargo test e2e::offline_all` |
| M0✓ | TRACE rows for A*, B1/B2/B6, C1/C3, D1/D2, E*, I*, K1/K4 flip `planned`→`done` | `./verify_meshwork.sh` |

### M1 — graph verbs, comments/attachments, prime, import (stop-line: session ritual switched)

| id | item (MW refs) | verify |
|---|---|---|
| 1.1 | `dep add`/`dep rm` (B1) | `cargo test e2e::dep_edit` |
| 1.2 | `tree`/`why`/`blocked`; `tree` renders 5-deep fixture chain with cosmetic level names (B8, C2) | `cargo test e2e::tree_why_blocked_golden` |
| 1.3 | Category segment-prefix + label queries (B4, B5) | `cargo test query::category_labels` |
| 1.4 | `comment` (author fallback chain) + `attach` (`--force`, >1MB lint warn) (K1–K3) | `cargo test e2e::comment_attach` |
| 1.5 | `prime`: byte-budgeted ≤6KB, measured on kitchen-sink fixture (D3, D5) | `cargo test e2e::prime_budget` |
| 1.6 | CLI-surface freeze test: `--help` output lists exactly DESIGN §6 — the non-goals fence, enforced (D4, §3 non-goals) | `cargo test e2e::cli_surface_frozen` |
| 1.7 | `import todo`: baseline checkbox format from committed sazed-format sample → golden (J3) | `cargo test e2e::import_todo_golden` |
| 1.8 | Gate §8 self-host activates: meshwork's own `meshwork/` initialized, its TODO items imported | `./verify_meshwork.sh` |
| 1.9 | **Manual pilot (sazed):** import real TODO.md; SessionStart hook injects `prime`; run 2 real sessions; `check-todo.sh` retired; HANDOFF.md → ≤2KB pointer | pilot checklist in sazed commit message; REQUIREMENTS §4 clauses 1+5 |

### M2 — portfolio (stop-line: leras joins)

| id | item (MW refs) | verify |
|---|---|---|
| 2.1 | `repos.toml` + `repos.local.toml` overrides + default-path resolution (G2) | `cargo test e2e::registry_overrides` |
| 2.2 | Portfolio union pipeline (`repo` column, one code path) + `portfolio q` (G1, G3) | `cargo test e2e::portfolio_union_golden` |
| 2.3 | Single-repo cross-repo resolution via by-ID glob; absent-repo scenario 6 (B3, G5) | `cargo test e2e::crossrepo_resolution e2e::absent_repo` |
| 2.4 | `sequence.md` + `portfolio next` total ordering (sequenced → repos.toml order → per-repo) (G4) | `cargo test e2e::portfolio_next_ordering` |
| 2.5 | Gate §7 perf: synthetic generators (seeded), 1K tasks / 20 repos, N≥7 median (C4) | `./verify_meshwork.sh` (§7) |
| 2.6 | **Manual:** leras registered; one real cross-repo `needs` sazed↔leras | `portfolio ready` shows it; REQUIREMENTS §4 clause 3 |

### M3 — mirror (stop-line: OEM-face visibility)

| id | item (MW refs) | verify |
|---|---|---|
| 3.1 | `mirror push` create path: marker search → adopt-or-create, labels, relationships-if-supported (H1, H3) | `cargo test e2e::mirror_create` |
| 3.2 | Append path: comment markers, transition comments, attachment blob links; never-mutate asserted by stub (zero edit/close/delete calls across the whole suite) (H1, H2) | `cargo test e2e::mirror_append e2e::mirror_never_mutates` |
| 3.3 | Idempotency + race-adopt: scenario 4; second push `.calls` golden empty (H3) | `cargo test e2e::mirror_idempotent` |
| 3.4 | `mirror status` drift report, read-only locally (H4) | `cargo test e2e::mirror_status_reports_only` |
| 3.5 | **Manual acceptance drill:** scratch GitHub repo, push twice, second is a no-op; externally close an issue, `mirror status` reports it, store untouched | REQUIREMENTS §4 clause 4 |

### M4 — doc drill-through (stop-line: v1)

| id | item (MW refs) | verify |
|---|---|---|
| 4.1 | `show --docs`: anchor-scoped excerpts, ~4KB/link cap (F1, F2) | `cargo test e2e::show_docs_excerpts` |
| 4.2 | Lint anchor validation (warn) (F3) | `cargo test lint::anchor_missing_warn` |
| 4.3 | **v1 acceptance:** `./verify_meshwork.sh --strict` (TRACE fully `done`) + all five REQUIREMENTS §4 clauses checked off | `./verify_meshwork.sh --strict` |

## 3. TRACE.md seed (requirement → named tests)

| req | tests |
|---|---|
| MW-A1 | `parse::roundtrip_hand_edited`, `e2e::add_show_roundtrip` |
| MW-A2 | `tables::memtable_no_disk`, `e2e::cache_delete_safe` |
| MW-A3 | `e2e::init_layout` (asserts nothing written outside repo, no hooks) |
| MW-A4 | `id::collision_reroll`, `e2e::merge_duplicate_id` |
| MW-A5 | `lint::description_size_warn`, `e2e::show_caps` |
| MW-A6 | `parse::unknown_field_warns` |
| MW-B1 | `tables::edge_kinds`, `e2e::dep_edit` |
| MW-B2 | `lint::cycle_needs`, `lint::cycle_parent` |
| MW-B3 | `e2e::crossrepo_resolution`, `lint::parent_crossrepo_error` |
| MW-B4 | `query::category_segment_prefix` |
| MW-B5 | `query::labels_orthogonal` |
| MW-B6 | `e2e::ready_golden` (incl. container-exclusion + unresolved-blocks cases) |
| MW-B7 | `lint::parent_rollup_warn` |
| MW-B8 | `e2e::tree_why_blocked_golden` (5-deep chain, cosmetic level names) |
| MW-C1 | `e2e::raw_sql_tables` (all five tables) |
| MW-C2 | `e2e::tree_why_blocked_golden` |
| MW-C3 | `e2e::json_stable_shapes` |
| MW-C4 | gate §7 (`perf::ready_1k`, `perf::portfolio_20x`, N≥7 median) |
| MW-D1 | `e2e::show_caps` + `e2e::ready_golden` (no verb dumps the store) |
| MW-D2 | `e2e::caps_and_more_marker` |
| MW-D3 | `e2e::prime_budget` (≤6144 bytes measured) |
| MW-D4 | `e2e::cli_surface_frozen` |
| MW-D5 | `unit::budget_bytes_not_lines` |
| MW-E1 | `e2e::transitions` |
| MW-E2 | `e2e::close_gating`, `e2e::close_waive_recorded` |
| MW-E3 | `e2e::log_append_on_transitions` |
| MW-E4 | `e2e::discovered_from_edge` |
| MW-F1 | `parse::docs_links` |
| MW-F2 | `e2e::show_docs_excerpts` |
| MW-F3 | `lint::anchor_missing_warn` |
| MW-K1 | `e2e::comment_attach` (author fallback), `parse::comment_format` |
| MW-K2 | `e2e::comment_attach` (roundtrip, `--force`) |
| MW-K3 | `lint::attachment_size_warn` |
| MW-K4 | `e2e::add_show_roundtrip` (last-3 + count) |
| MW-G1 | `e2e::repo_self_contained` (alpha cloned alone; all single-repo verbs work) |
| MW-G2 | `e2e::registry_overrides` |
| MW-G3 | `e2e::portfolio_union_golden` |
| MW-G4 | `e2e::portfolio_next_ordering` |
| MW-G5 | `e2e::absent_repo` |
| MW-H1 | `e2e::mirror_create`, `e2e::mirror_append` |
| MW-H2 | `e2e::mirror_never_mutates` |
| MW-H3 | `e2e::mirror_idempotent` (incl. race-adopt) |
| MW-H4 | `e2e::mirror_status_reports_only` |
| MW-H5 | `e2e::offline_all` |
| MW-I1 | `e2e::merge_concurrent_worktrees` |
| MW-I2 | `e2e::merge_union_poison`, `e2e::invalid_visible` |
| MW-J1 | gate §§1–2 + Cargo.toml review at B0 (dep allowlist in the script) |
| MW-J2 | gate §§4–5 |
| MW-J3 | `e2e::import_todo_golden` + M1.9 pilot checklist |
| MW-J4 | `fixtures::corpus_covers_features` |
| MW-J5 | gate §6 (trace completeness, `--strict` at v1) |
| MW-J6 | `e2e::offline_all` + stub-gh harness (no test may touch the network; gate runs with proxy vars cleared) |

## 4. What "go" does NOT include

Everything in REQUIREMENTS §3 (non-goals), plus: no MCP server, no CI service (the gate is local, house pattern), no publishing to crates.io, no README marketing. Scope changes route through an owner ruling amending REQUIREMENTS §3 — not through this plan.
