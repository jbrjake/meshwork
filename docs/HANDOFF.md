# HANDOFF (≤2KB — pointer, not narrative; durable state lives in PLAN Position + TRACE + commits)

**Done:** Through PLAN 0.5 (2026-08-04). `add`: all DESIGN §6 flags, field order matches the normative example, `write::yaml_scalar` quotes what YAML would misread (trailing `::`, `: `, bool/number lookalikes, commas), refuses cross-repo `--parent` at creation, prints id on line 1; `MESHWORK_ID_SEED` → `IdGen::from_seed_str`, `created:` from `clock::today()` (UTC, `MESHWORK_TODAY` override for byte-stable goldens). `show`: full task view, last-3 comments + `… and N more (use --comments)`, `--comments` for all, invalid files render loud (text → error exit; `--json` → status:"invalid" + error). TRACE: MW-A1, MW-E4, MW-K4 → done (with e2e::discovered_from_edge checking the edge in SQL too).

**Decisions:** `show` renders log fully (spec caps only comments); listing caps (20 rows) land with `ready` (0.8). `e2e::caps_and_more_marker` (MW-D2) also lands 0.8 alongside listing caps.

**Open threads:** MW-J4 planned until --bless (0.8); fixtures.rs 510 (warn).

**Next concrete step:** PLAN 0.6 — transitions `start/block/drop/reopen` + log append; `block` demands `--reason` (E1, E3). Status edits touch ONE frontmatter line (MW-I1 — no full-file rewrite).
verify: `cargo test e2e::transitions` exits 0.
