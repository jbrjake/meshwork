# HANDOFF (≤2KB — pointer, not narrative; durable state lives in PLAN Position + TRACE + commits)

**Done:** B0–B3 (2026-08-04): bootstrap + gate green; fixture corpus committed per DESIGN §13 — alpha (33 tasks: 5-deep chain, every status/edge kind, cross-repo→beta, absent→gamma, good+bad doc anchors, multi-author comments w/ continuations, >1MB attachment, seq gaps, no-verify task), alpha-broken (all 9 failure modes), beta, portfolio (repos.toml + sequence.md), golden/ (populates per-feature via --bless). `fixtures::corpus_covers_features` enforces corpus completeness text-level (deliberately parser-free). Gate §4 coverage is now live.

**Decisions:** pre-build decisions in DESIGN §15. New, corpus-derived: `verify:` values ending in `::` (cargo test filters) must be YAML-quoted — a trailing colon at EOL is a YAML mapping indicator; the strict parser (0.1) must reject unquoted ones like any other bad YAML. Fixture doc anchors use heading-slug convention: `#§-budget-path` ↔ heading `## § budget path` (lowercase, spaces→dashes).

**Open threads:** TRACE MW-J4 stays `planned` until golden byte-compare + `--bless` machinery exists (0.8); the corpus test alone doesn't satisfy MW-J4's full text.

**Next concrete step:** PLAN item B4 — stub `gh`: `tests/bin/gh` records argv+stdin to `.calls`, replays canned JSON from `tests/canned/`; harness prepends to `$PATH`.
verify: `cargo test stub_gh::` exits 0.
