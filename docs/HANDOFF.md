# HANDOFF (≤2KB — pointer, not narrative; durable state lives in PLAN Position + TRACE + commits)

**Done:** Through PLAN 0.10 (2026-08-04). Merge scenarios 1–3 green with real git (bare origin + two configured clones, origin HEAD pinned to main for deterministic clones): (1) concurrent clones close/create/comment on the same task → union attr merges markerless, both comments survive, lint clean; (2) `MESHWORK_ID_SEED` forces the same mint in both clones → lint reports duplicate-id, `--fix` re-slugs the later side, references resolve to the keeper, lint exits 0; (3) conflicting status edits union into a dup key → row surfaces invalid in `q`, `--fix` repairs to one status line with both log entries intact. TRACE: A4, I1, I2 done (30 planned).

**Decisions:** merge tests seed the `## comments` section at base so both sides purely append (avoids union duplicating the heading — a known cosmetic artifact when both sides create the section; parser tolerates it either way).

**Open threads:** none new.

**Next concrete step:** PLAN 0.11 — offline-everything scenario 5 (H5): `$PATH` without `gh`, no network — every non-mirror verb runs clean.
verify: `cargo test e2e::offline_all` exits 0. Then M0✓ TRACE sweep.
