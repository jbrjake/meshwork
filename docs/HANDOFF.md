# HANDOFF (≤2KB — pointer, not narrative; durable state lives in PLAN Position + TRACE + commits)

**Done:** B0→1.8 in one session (2026-08-04). M0 complete; M1 complete through 1.8. Self-host is LIVE: this repo's own `meshwork/` store (alias `mw`) holds the 15 remaining plan items as tasks, needs-chained in plan order — `meshwork ready` shows exactly the next item (mw-ntt5, the 1.9 pilot), `prime` is 82B. Gate: ALL SECTIONS PASS with §8 active; only §7 (perf) still SKIPs, pending 2.5. TRACE: 17 planned rows, every one mapped to an M2+/manual test.

**Decisions:** per-commit messages carry the rationale; determinism hooks are MESHWORK_ID_SEED / MESHWORK_TODAY / MESHWORK_BLESS. Post-M1 rule now in force (CLAUDE.md override 1): new work → `meshwork add`, not plan edits.

**Open threads:** 1.9 is the stop-line and it is OWNER-DRIVEN: import sazed's real TODO.md, SessionStart hook injecting `prime`, 2 real sessions, retire check-todo.sh. No further plan item may start before it (plan ordering rule). When M2 resumes: 2.3 will re-bless ready-alpha.json (az-x9b2 becomes ready — expected diff).

**Next concrete step:** PLAN 1.9 — manual sazed pilot; checklist in the sazed commit message per REQUIREMENTS §4 clauses 1+5. Dev resumes at 2.1 afterward (`meshwork show mw-5ckb`).
