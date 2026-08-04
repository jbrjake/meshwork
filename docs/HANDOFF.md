# HANDOFF (≤2KB — pointer, not narrative; durable state lives in PLAN Position + TRACE + commits)

**Done:** Through PLAN 0.9 (2026-08-04). `src/lint.rs`: sorted/deduped findings with stable codes — errors: parse, duplicate-key, duplicate-id, cycle-needs/parent, parent-crossrepo, blocked-no-reason, dangling (needs/parent); warnings: unknown-key/schema, no-verify, dangling (soft kinds), description-size (>2KB), file-size (>64KB §15.5), attachment-missing/-size (>1MB), parent-rollup. `lint --fix` repairs ONLY mechanical damage: dup keys keep-first + logged repair entry; dup IDs re-slugged (keeper = earliest created then filename — content can't attribute inbound refs, so they're reported, never rewritten blind). lint-broken.json golden committed. TRACE: A5, B2, B7, K3 done (33 planned).

**Decisions:** lint exit 1 iff errors; warnings never block. Cross-repo needs/relates are skipped by single-repo lint (registry owns them, M2). Anchors check lands at 4.2 per plan.

**Open threads:** MW-B3 waits on e2e::crossrepo_resolution (2.3); MW-I2 waits on e2e::merge_union_poison (0.10).

**Next concrete step:** PLAN 0.10 — merge scenarios 1–3: concurrent-worktrees (union attr, both comments survive), duplicate-id-merge (seeded RNG forces collision; lint --fix re-slugs), union-poison (conflicting status lines → dup key → invalid row → --fix repairs) (I1, I2, A4).
verify: `cargo test e2e::merge_` exits 0.
