# HANDOFF (≤2KB — pointer, not narrative; durable state lives in PLAN Position + TRACE + commits)

**Done:** B0→1.8 in one session (2026-08-04); M1 complete through 1.8; gate ALL SECTIONS PASS (§7 perf SKIPs until 2.5). 2026-08-06: `docs:` refs backfilled onto all tasks (mw-g4a9); session ritual flipped to `meshwork prime` with a SessionStart hook in `.claude/settings.json`, proven live in a fresh session (mw-6tzr); MIT LICENSE (Jonathon Rubin); adoption skill authored at `~/.claude/skills/meshwork` (machine-local, not in this repo).

**Decisions:** per-commit messages carry rationale; determinism hooks MESHWORK_ID_SEED / MESHWORK_TODAY / MESHWORK_BLESS. Post-M1 rule in force: new work → `meshwork add`, not plan edits. Owner ruling 2026-08-06: NO global cargo install — each repo pins its own meshwork version; binaries from tag-push GitHub Actions releases (mw-der3).

**Open threads:** 1.9 (sazed pilot, mw-ntt5) now `needs: mw-der3` — distribution must exist before another repo adopts. `ready` = mw-der3 (GitHub remote + release.yml + first tag). When M2 resumes: 2.3 re-blesses ready-alpha.json (az-x9b2 becomes ready — expected diff).

**Next concrete step:** mw-der3, then the 1.9 manual pilot (checklist in sazed commit per REQUIREMENTS §4 clauses 1+5), then dev resumes at 2.1 (`meshwork show mw-5ckb`).
