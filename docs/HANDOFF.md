# HANDOFF (≤2KB — pointer, not narrative; durable state lives in PLAN Position + TRACE + commits)

**Done:** M0 complete + PLAN 1.1 (2026-08-04). `dep add/rm <a> --needs <b>`: one-line needs edits without opening the file; refuses self-deps, duplicates, and dangling same-repo targets (cross-repo `repo#id` passes through — registry's business); removing the last entry drops the key line (`edit::remove_scalar`). Edges verified down to the SQL tables; ready reflects them immediately. TRACE: MW-B1 done (26 planned).

**Decisions:** dep edits write no log entry — git history records them; the log stays for lifecycle + notes. Cycle prevention stays at lint (MW-B2 says lint-time), only local guardrails at edit time.

**Open threads:** MW-K1 flips at 1.4 (comment_attach).

**Next concrete step:** PLAN 1.2 — `tree`/`why`/`blocked`; `tree` renders the 5-deep fixture chain with cosmetic level names (B8, C2).
verify: `cargo test e2e::tree_why_blocked_golden` exits 0.
