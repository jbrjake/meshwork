# HANDOFF (≤2KB — pointer, not narrative; durable state lives in PLAN Position + TRACE + commits)

**Done:** Through PLAN 1.5 (2026-08-04). `prime`: counts line · ready top-10 (normative SQL — one queue truth) · in-progress with last log line · blocked with reasons · loud invalid alarm. Byte discipline: 160B per-line clamp (`write::clamp_bytes`, char-boundary safe, visible `…`), 6144B hard budget with an explicit truncation tail — proven on the corpus AND a hostile 120-task store with giant titles. `unit::budget_bytes_not_lines` pins bytes-not-lines (MW-D5). TRACE: D3, D5 done (18 planned).

**Decisions:** prime JSON mirrors rows (ready capped at 10) but the byte budget governs the TEXT artifact — that's what the SessionStart hook injects and what gate §8 measures.

**Open threads:** none new.

**Next concrete step:** PLAN 1.6 — CLI-surface freeze test: `--help` lists exactly DESIGN §6 (D4, §3 non-goals fence). Note: `show --docs` flag must exist in the surface but its behavior lands at 4.1 — until then it errors honestly.
verify: `cargo test e2e::cli_surface_frozen` exits 0.
