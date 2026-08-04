# HANDOFF (≤2KB — pointer, not narrative; durable state lives in PLAN Position + TRACE + commits)

**Done:** Through PLAN 0.3 (2026-08-04). `src/store.rs`: `load_repo` reads config.toml (alias/default_author/[hierarchy]/mirror; repo name = dir name, matching registry default) + filename-sorted task entries, parse failures ride along. `src/tables.rs`: `session_for(&[RepoStore])` registers the five-table contract — tasks (incl. `waived`, `error`, status='invalid' rows), edges (child→parent direction, `resolved` = dst in loaded set; registry lookup deferred to 2.3), labels, comments (1-based `ord`), repos. Union = same fn with N stores (MW-G3 pinned by test).

**Decisions:** `github` column is Int64 (SQL ergonomics); `error` column on tasks carries invalid-row diagnostics (satisfies "error text attached", DESIGN §3). Loaded repos' `remote` is NULL until the registry (2.1).

**Open threads:** MW-J4 planned until --bless (0.8); fixtures.rs at 510 (warn). Clippy pedantic runs deny in gate — doc_markdown wants backticks on `DataFusion` etc.; write docs accordingly.

**Next concrete step:** PLAN 0.4 — `init`: writes `meshwork/` layout, config.toml, `.gitattributes` (`tasks/*.md merge=union`), `.cache/.gitignore`; refuses outside a git repo (A3, I1).
verify: `cargo test e2e::init_layout` exits 0.
