# REQUIREMENTS-meshwork.md

**Status: DESIGN PHASE ONLY — build not authorized (owner, 2026-08-04). This doc + DESIGN-meshwork.md are the deliverable; no repo, no code.**
**Provenance:** four-agent session-history audit of leras (102 sessions) + sazed (132 sessions), giles machinery review, and Aug-2026 landscape survey, 2026-08-04. Owner rulings same day: `br`/beads out (bus-factor-1, SQLite dep), no database of any kind, store = human-readable markdown in git, GitHub = view never store, nerdsnipe knowingly accepted. Name: "taskmesh" was squatted (crypto experiment + PHP SDK + X handle); owner ruled adjacent-unused — **meshwork** verified clean in the dev-tool space 2026-08-04.
**Rev 2 (owner corrections, 2026-08-04):** comments allowed (self-professed identities, no accounts) · attachments allowed · saga/epic/sprint/story-depth hierarchy supported generically with zero agile semantics · GitHub push widened to issues+comments+attachments but strictly append-only (no state changes, no deletes).
**Rev 3 (adversarial review fixes, 2026-08-04):** same-file concurrent appends made mergeable via committed `merge=union` attributes (the §4 gate was unsatisfiable without it) · mirror idempotency moved off merge-fragile frontmatter counters onto comment-hash markers read back from the issue · `ready` excludes containers with live children (deep hierarchies were going to flood it) · cross-repo `needs` resolution defined for single-repo commands · ID-collision claim corrected (parallel clones CAN collide; lint owns the remedy) · all budgets restated in bytes (a line target had crept back into MW-A5) · `parent` edges ruled same-repo · YAML-parser clause added to MW-J1 (`serde_yaml` is archived and fails it).
**Rev 4 (production-verification hardening, 2026-08-04):** committed e2e fixture corpus, per-requirement test trace, and one offline gate script are now MUST (MW-J4–J6) · every open question resolved to a decision (DESIGN §15) · `PLAN-meshwork-build.md` added — ordered work items each with a `verify:` command; "go" executes it top-to-bottom with nothing left to decide.

meshwork is a Rust CLI that stores a project's task graph as markdown-with-frontmatter files in the repo's own git tree, answers SQL over that graph with no database (sahjhan's DataFusion-over-JSONL pattern), shows agents only what they need at the moment they need it, links tasks outward to design docs, links across repos to form one distributed portfolio graph, and pushes an append-only view to GitHub Issues.

## 1. Evidence (why this exists)

| Measured, 2026-08-04 | sazed | leras |
|---|---|---|
| All-time Edit calls targeting TODO.md+HANDOFF.md | 19.75% (927/4,693) | ~16% (641) |
| Sessions reading both files in first 5 tool calls | ~95% (~22K-token tax) | ~90% |
| Worst line-cap thrash episode | 34% of a session's Bash calls; 14 gate failures in 84 min | 26 consecutive tool calls |
| Cap raises (always reactive, never structural) | 500→550 (2026-08-04, `bc590a7`) | 200→300→450→600 in 8 days |

Root cause: a task **graph** stored as prose, gated by a proxy metric (`wc -l`) that gets fought and gamed (leras CLAUDE.md: "131 lines", 38KB). Full findings in session memory `agent-task-tracker-investigation`.

## 2. Requirements

Keywords MUST / SHOULD / MAY per RFC 2119. IDs are stable; cite as `MW-A1`.

### A. Store

- **MW-A1 (MUST)** Canonical store = one markdown file per task with YAML frontmatter, under `meshwork/tasks/` in the owning repo's git tree. Human-readable, hand-editable in any editor; the tool MUST tolerate and re-validate hand edits. (Backlog.md's concept, our schema.)
- **MW-A2 (MUST)** No database. No SQLite, no Dolt, no server, no daemon, no persistent index as source of anything. Queries execute in-memory per invocation (DataFusion `MemTable`, exactly `sahjhan/src/query/mod.rs`). A derived JSONL projection MAY exist as a gitignored, regenerable cache; deleting it is always safe.
- **MW-A3 (MUST)** No git hooks installed by default; nothing written outside the repo (and the portfolio repo, §G). Explicit lesson from beads' hook-hijack reports.
- **MW-A4 (MUST)** Task IDs: short repo-prefixed slugs with a random component (e.g. `sz-k7f3`), collision-checked against local files at creation. Parallel clones share no state and CAN mint the same ID (rare — ~1M combinations, small unmerged windows — but possible), so `lint` MUST detect duplicate IDs post-merge and `lint --fix` MUST re-slug the side with fewer inbound edges, rewriting same-repo references (cross-repo inbound references are reported, never silently rewritten). Never renumbered, never reused.
- **MW-A5 (MUST)** Bounded context by construction: one task per file; the description section targets ≤~2KB (≈500 tokens — a byte budget, per MW-D5; a line target here would re-open the exact gaming hole §1 documents) with long design detail behind `docs:` links (§F). Append-only sections (log, comments) may grow, but read-time output is always capped (§D) — context cost is controlled at read time, storage is history.
- **MW-A6 (SHOULD)** All frontmatter fields validated against a strict schema by `meshwork lint`; unknown fields warn.

### B. Graph model

- **MW-B1 (MUST)** Typed edges: `needs` (hard dependency; blocks readiness), `parent` (nesting/subtask, arbitrary depth), `discovered-from` (provenance), `relates` (soft). Work is a graph, not a list.
- **MW-B2 (MUST)** Cycle detection on `needs` + `parent` at lint time; cycles are errors.
- **MW-B3 (MUST)** Cross-repo edges by git repo name: `sazed#sz-k7f3` is addressable from any registered repo (§G). `needs`, `relates`, and `discovered-from` MAY cross repos; `parent` MUST NOT (lint error) — hierarchy is per-repo, and portfolio-level grouping lives in `sequence.md` tranches (MW-G4). Single-repo commands MUST still resolve cross-repo `needs` targets through the registry (direct by-ID lookup, DESIGN §5): a dependency on a `done` task in a sibling repo must not read as blocking merely because the current command is single-repo.
- **MW-B4 (MUST)** Hierarchical categories: one slash-path per task (e.g. `engine/spill/compaction`), arbitrary depth; queries match by whole-segment prefix (`engine/spill` matches `engine/spill/compaction`, never `engine/spillover`).
- **MW-B5 (MUST)** Cross-cutting labels: flat, many per task, orthogonal to categories.
- **MW-B6 (MUST)** `ready` = status `open` ∧ every `needs` target is `done`/`dropped` ∧ no child (via `parent`) is `open`/`doing`/`blocked`. The child clause keeps MW-B8 hierarchies from flooding the queue with saga/epic containers — a container with live children is not actionable; a parent whose children are all closed is (residual work, then close). This is the core queue primitive; its SQL definition is normative (DESIGN §5).
- **MW-B7 (SHOULD)** Parent rollup: a parent is not `done` while open children exist (lint warning, not auto-close).
- **MW-B8 (MUST)** Deep-hierarchy usability with zero method semantics: a user MUST be able to run a saga → epic → sprint → story → task structure purely with category depth and/or `parent` nesting. Config MAY assign display names to category depths (e.g. `levels = ["saga","epic","sprint","story"]`) — cosmetic only. The tool knows nothing about sprints, agile, velocity, or ceremonies, ever (those stay in giles).

### C. Query

- **MW-C1 (MUST)** Real SQL: `meshwork q "SELECT …"` over virtual tables `tasks`, `edges`, `labels`, `comments`, `repos` (DataFusion dialect). No bespoke query language.
- **MW-C2 (MUST)** Canned verbs that expand to defined SQL: `ready`, `blocked`, `tree <id>`, `why <id>` (walks the blocking chain and says what unblocks it).
- **MW-C3 (MUST)** Every command supports `--json` with a stable, versioned shape.
- **MW-C4 (SHOULD)** Cold `ready` ≤100ms at 1,000 tasks/repo on the owned machines; portfolio union over ~20 repos ≤1s.

### D. Context discipline (the actual point)

- **MW-D1 (MUST)** Flat two-level disclosure: capped one-line-per-task queue views → full single-task view by ID. No command in the agent path dumps the whole store. (Empirical basis: flat beats hierarchical, arXiv:2607.17598.)
- **MW-D2 (MUST)** Every listing capped by default (20 rows; last 3 comments) with an explicit `… and N more` marker; `--all` is opt-in. (giles SessionStart pattern, generalized.)
- **MW-D3 (MUST)** `meshwork prime`: a ≤6KB session-start digest (≈1.5K tokens at the normative approximation of 4 bytes/token — the gate measures bytes) — ready top-N, in-progress, blocked-with-reason — suitable for SessionStart-hook injection. This replaces reading TODO.md+HANDOFF.md at session start (the ~22K-token tax).
- **MW-D4 (MUST)** CLI-first; no MCP server in v1 (CLI ≈1–2K tokens vs 10–50K for MCP schemas).
- **MW-D5 (SHOULD)** Output budgets measured in bytes/approx-tokens, never lines — closes the `wc -l` gaming hole permanently.

### E. Lifecycle & discipline (carried over from CLAUDE-BASELINE)

- **MW-E1 (MUST)** Statuses: `open`, `doing`, `blocked` (must name blocker + unblock condition), `done`, `dropped`.
- **MW-E2 (MUST)** Every task carries a `verify:` command (set at creation via `add --verify` or by hand-edit; missing `verify:` is a lint warning while open and forces `close` to demand `--waive`); `meshwork close <id>` runs it via `sh -c` from the repo root, records exit code + date in the log, and closes only on exit 0. `--waive "<reason>"` is recorded, loud, and queryable. Can't write a verify → the task isn't ready; split out a spike whose deliverable IS the verify (baseline rule, preserved verbatim).
- **MW-E3 (MUST)** Append-only per-task log (dated transitions + one-line notes) in the task body — the durable handoff record, replacing HANDOFF.md narrative.
- **MW-E4 (SHOULD)** `--from <id>` at creation records `discovered-from` provenance; kills the "audit filed in the wrong file, invisible for a day" routing failure.

### F. Wiki / doc drill-through

- **MW-F1 (MUST)** `docs:` frontmatter: repo-relative paths with optional `#§-anchors` (baseline's stable-anchor convention) linking a task to its design detail.
- **MW-F2 (MUST)** `meshwork show <id> --docs` resolves links and emits anchor-scoped **excerpts** (capped), not whole files. Drill-through is itself progressive disclosure.
- **MW-F3 (SHOULD)** Lint validates that linked anchors exist (warn).

### K. Comments & attachments (owner correction, rev 2)

- **MW-K1 (MUST)** Append-only comments per task. Each comment carries a date and a **self-professed identity** — a free author string (`jon`, `claude/f10a7561`, …). No accounts, no auth, no verification; identity is a claim, recorded as claimed.
- **MW-K2 (MUST)** Attachments: arbitrary files (log excerpts, artifacts, images) stored under `meshwork/attachments/<task-id>/` in the repo tree, referenced from task frontmatter and addressable from comments. They live in git like everything else.
- **MW-K3 (SHOULD)** Lint warns on attachments >1MB (suggest excerpting; keep full logs out of git history when a 50-line excerpt carries the signal).
- **MW-K4 (MUST)** Comments obey §D at read time: default `show` renders the last 3 with a count; `--comments` renders all; the `comments` SQL table exposes everything.

### G. Portfolio (the distributed graph)

- **MW-G1 (MUST)** Per-project scope by default; a repo's tasks travel with the repo — clone, branch, merge like code. Any clone is complete for its repo. No central server, ever.
- **MW-G2 (MUST)** A repo registry in a small dedicated **portfolio git repo**. The committed `repos.toml` maps name → GitHub remote; local paths resolve to `~/Documents/code/<name>` by default, overridable per machine in a gitignored `repos.local.toml` — committed absolute paths would break on the second owned machine. Note: the current code root is in no repo and has no backup (STATUS.md rule) — the registry MUST NOT live there loose.
- **MW-G3 (MUST)** `meshwork portfolio <verb>` unions all registered repos into one graph (every table gains a `repo` column); cross-repo `needs` resolve; `portfolio q` gives cross-repo SQL.
- **MW-G4 (MUST)** Master sequencing: a human-readable ordered file in the portfolio repo overlays global priority across repos; `portfolio next` respects it; unsequenced tasks fall back to `repos.toml` order, then per-repo order — the full ordering is total and deterministic. Resequencing = editing that one file.
- **MW-G5 (MUST)** Graceful degradation: an unregistered/absent repo makes its cross-repo edges *unresolved* — reported, and conservatively treated as blocking; never an error, never data corruption.

### H. GitHub push (append-only view, never store — rev 2)

- **MW-H1 (MUST)** One-way, **append-only** push per repo via `gh`: create issues; post comments (task comments, and status transitions / local edits surfaced *as comments*); expose attachments (as links to the committed files on the remote — they're already in git). Opt-in per repo.
- **MW-H2 (MUST)** Never mutate: no closing, reopening, or deleting issues; no editing issue state, title, body, or labels after creation. A locally-dropped task gets a comment saying so; the GitHub issue stays as a human decision.
- **MW-H3 (MUST)** Idempotent without merge-fragile state: the mirror issue number lives in frontmatter (set once, never changes — merge-safe). Pushed comments carry an invisible content-hash marker; `mirror push` lists the issue's existing markers and posts only unmarked local comments — no pushed-comment counters in frontmatter (a counter merges wrong when two clones interleave comments). Issue creation first searches for the task-ID marker, so two clones racing `mirror push` cannot create two issues. Re-push after no changes is a no-op. Reading the mirror for idempotency is not reading it as authority (MW-H4 intact).
- **MW-H4 (MUST)** GitHub is never read as authority. `mirror status` MAY report drift (e.g. externally closed issues) — as a report for a human, never an auto-import.
- **MW-H5 (MUST)** Every core command works offline with zero network; only `mirror` touches the network.

### I. Concurrency & merge

- **MW-I1 (MUST)** Safe under concurrent Claude sessions in separate clones/worktrees: file-per-task makes creation conflict-free; status edits touch one frontmatter line; logs and comments are append-only at file end. Append-only is NOT sufficient for vanilla git merge — two sides appending at the same end-of-file is a textbook conflict — so `init` MUST commit a `.gitattributes` marking `tasks/*.md merge=union` (a git built-in, zero per-clone setup, not a hook: MW-A3 intact). Known cost, accepted: truly conflicting edits to the same frontmatter line union-merge into duplicate YAML keys instead of conflict markers; strict parsing rejects the file and lint repairs it (MW-I2).
- **MW-I2 (MUST)** No locks, no daemon; conflicts resolve via normal git merge; `meshwork lint` detects structural damage post-merge (duplicate frontmatter keys, duplicate IDs, dangling edges) and `--fix` repairs it. A file that fails to parse MUST surface as a loud error in every command that would have read it — an unparseable task must never silently vanish from queries.

### J. Non-functional

- **MW-J1 (MUST)** Rust, single static binary. Dep posture mirrors sahjhan (clap, serde, serde_json/yaml, datafusion, toml, thiserror; tokio only because DataFusion needs the runtime). No SQLite. No unmaintained hobby deps on the critical path — which rules on the YAML frontmatter parser too: `serde_yaml` is archived (2024) and does not qualify. Decision: `serde_yaml_ng` (maintained continuation, drop-in serde API); fallback `saphyr` if it stalls.
- **MW-J2 (MUST)** meshwork's own code obeys CLAUDE-BASELINE house numbers (500 target / 750 ceiling, 80% coverage, N≥7 bench reps where relevant).
- **MW-J3 (SHOULD)** Buildable/adoptable in one session per repo including TODO.md import (DESIGN §9).
- **MW-J4 (MUST)** Committed end-to-end fixture corpus (DESIGN §13): golden repos + a portfolio exercising every feature and every failure mode this spec names — deep hierarchy, every status, every edge kind, cross-repo and absent-repo edges, cycles, duplicate IDs, unparseable files, blocked-without-reason, oversized attachments, comment merges. E2E tests run the real binary against tempdir copies and compare `--json` output to committed golden files byte-for-byte; goldens regenerate only via an explicit `--bless` + reviewed git diff, never silently.
- **MW-J5 (MUST)** Traceable verification: every MW-* MUST requirement maps to ≥1 named test in an in-repo `TRACE.md`; `verify_meshwork.sh` (DESIGN §14) is the single gate — fmt, clippy `-D warnings`, unit+integration+e2e, coverage ≥80%, file caps 500/750, trace completeness — one exit 0, house pattern (verify_alpha.sh precedent). An untraced requirement fails the gate, not a review vibe.
- **MW-J6 (MUST)** The gate runs with zero network: `mirror` tests execute against a stub `gh` on `$PATH` that records invocations and replays canned responses. The live scratch-repo drill in §4 is a manual acceptance step outside the gate.

## 3. Non-goals (normative — this list is the anti-Jira, anti-nerdsnipe contract)

No web UI. No daemon or background sync. No user accounts, auth, roles, or permissions — identities are self-professed strings (MW-K1). No notifications. No time tracking, estimates, or burndown. No sprint/agile semantics — hierarchy is generic structure only (MW-B8); ceremonies stay in giles. No MCP server in v1. No plugin system. No bespoke query language. No SaaS component. GitHub is never the store and is never mutated beyond append (MW-H2). No two-way sync. Feature requests landing in this list are rejected by default; moving an item out requires an owner ruling recorded here.

## 4. Acceptance gate for v1

`meshwork prime` in a migrated sazed replaces the session-start read of TODO.md+HANDOFF.md at ≤1.5K tokens; two concurrent sessions create tasks, append comments to the *same* task, and close tasks in separate worktrees, then merge without manual conflict resolution; `portfolio ready` unions sazed+leras with a working cross-repo `needs` edge; a `mirror push` creates issues and appends comments on a scratch GitHub repo twice with the second run a no-op; `check-todo.sh` is retired in the pilot repo; a forced duplicate-ID collision merged across worktrees is caught by `lint` and repaired by `lint --fix`; and `verify_meshwork.sh` (DESIGN §14) exits 0 — coverage ≥80%, every MW-* MUST traced to a passing named test (MW-J5). Each requirement's verify command is already named in `PLAN-meshwork-build.md`, written before any code (baseline rule applies to this project too).
