# DESIGN-meshwork.md

**Status: DESIGN PHASE ONLY — build not authorized (owner, 2026-08-04).** Companion to REQUIREMENTS-meshwork.md (cited as `MW-*`). This doc says *how*; requirements say *what/why*. Rev 2 folds in the owner corrections: comments, attachments, generic deep hierarchy, append-only GitHub push. Rev 3–4 (2026-08-04): adversarial-review fixes (union merge, marker-based mirror idempotency, container-aware `ready`, single-repo cross-repo resolution, byte budgets) + test architecture (§13), gate (§14), and all open questions resolved to decisions (§15); the ordered build lives in `PLAN-meshwork-build.md`.

Shape in one paragraph: markdown-with-frontmatter task files in each repo's git tree are parsed per invocation into in-memory Arrow tables and queried with DataFusion SQL — the sahjhan pattern (`sahjhan/src/query/mod.rs`: rows → `MemTable` → `SessionContext`), no database anywhere. Canned verbs (`ready`, `why`, `prime`) are frozen SQL over those tables. The portfolio view is the same pipeline fed by N repos with a `repo` column. GitHub mirroring is an append-only projection at the very edge.

## 1. On-disk layout (per repo)

```
docs/meshwork/           # store root (owner-ruled 2026-08-06, mw-acgp; was meshwork/tasks/ — old stores migrate via git mv)
  .gitattributes         # "/*.md" + "/archive/*.md" merge=union — committed by init (MW-I1)
  config.toml            # repo alias ("sz"), format marker (absent = 1; newer than the binary knows = loud refusal, mw-n6nvzpa), defaults, default_author, mirror opt-in, level names
  sz-k7f3-spill-cliff.md # LIVE task files, flat in the store root — no tasks/ level
  archive/               # terminal (done/dropped) tasks — moved automagically on close/drop,
    sz-m0v3d-done-thing.md   # moved back on reopen; ALWAYS loaded, fully queryable (mw-45e2qf4)
  attachments/
    sz-k7f3/spill-p99.log          # MW-K2; lint warns >1MB (MW-K3)
  .cache/                # gitignored
    tasks.jsonl          # derived projection; delete-safe (MW-A2)
```

`config.toml` may name category depths, display-only (MW-B8):

```toml
[hierarchy]
levels = ["saga", "epic", "sprint", "story"]   # cosmetic labels; zero semantics
```

Portfolio repo (new, tiny, e.g. `~/Documents/code/portfolio/` — must be a real git repo, MW-G2):

```
repos.toml               # committed: name -> github remote; paths default to ~/Documents/code/<name>
repos.local.toml         # gitignored: per-machine path overrides (MW-G2)
sequence.md              # master sequencing overlay (MW-G4)
```

## 2. Task file format (normative example)

```markdown
---
id: sz-k7f3
title: Fix spill cliff at 600M keys
status: doing            # open | doing | blocked | done | dropped
category: engine/spill   # one slash-path, arbitrary depth (MW-B4/B8)
labels: [perf, p0]       # many, flat (MW-B5)
needs: [sz-a2m9, leras#lr-x4x1]   # hard deps; repo#id crosses repos (MW-B3)
parent: sz-b881          # nesting (MW-B1)
discovered-from: sz-c7q2 # provenance (MW-E4)
verify: cargo test -p sazed-spill -- --exact spill::cliff_600m
docs:
  - docs/PLAN-spill-durable-unification.md#§-budget-path   # drill-through (MW-F1)
attachments:
  - attachments/sz-k7f3/spill-p99.log                      # MW-K2
seq: 40                  # per-repo order weight; portfolio overlay supersedes
github: 214              # mirror issue number; set once, absent until mirrored (MW-H3)
created: 2026-08-04
blocked-reason:          # required non-empty iff status: blocked (MW-E1)
---
Why/context, written to act cold — a few lines, not a narrative.
Acceptance beyond `verify:` if any. Long design lives behind docs:, never here (MW-A5).

## log
- 2026-08-04 open→doing — repro landed, bisecting spill batch size

## comments
- 2026-08-04 [jon] p99 only degrades with governed-spill on; see spill-p99.log
- 2026-08-04 [claude/f10a7561] bisected to batch=64k; excerpt attached
```

Format decisions: status is a single frontmatter line (one-line diffs, MW-I1); `## log` and `## comments` are append-only at end-of-file, one bullet per entry — `- <date> [<author>] text`, continuation lines indented two spaces, anything bigger becomes an attachment (MW-K1). Same-file concurrent appends merge cleanly via the committed `merge=union` attribute (MW-I1); union's failure mode — conflicting edits to one frontmatter line become duplicate YAML keys — is rejected by strict parsing and repaired by `lint --fix`. `blocked` without `blocked-reason` is a lint error. Minted stamps — new log lines, comment lines, `created:` — carry UTC minute resolution (`2026-08-06T21:47Z`; mw-zp1h12d, a §15.8 minting rule): union-merged same-day appends keep a recoverable order and the mirror's comment hash (§8) stops colliding on same-day identical text. The parser accepts date-only forever; display may keep date-only; last-activity is always derived as the max stamp in the file (a stored `updated:` field is rejected — one shared frontmatter line is a union-merge hotspot). Log lines have a normative grammar (mw-3wnhhvp): a minted entry is either a transition — `- <stamp> <from>→<to>[ — <note>]`, from/to drawn from the five statuses, the note separator a space-wrapped em dash — or free text (`- <stamp> created`, `- <stamp> close attempt — verify exit <N>`); every minted form fits, including block's reason, `start`'s `— claimed by <author>` suffix, and close's `— verify exit 0` / `— waived: <reason>`. Parsing is positional and never validates history: token one is the date as written, a second token reading `<status>→<status>` makes the entry a transition, anything else stays free text — old date-only and free-text lines are legal forever, and a note written without the em dash still counts (the grammar binds minting, not history). This grammar is exactly what §4's `log` table projects: from/to columns on transitions, NULL on free text. `claimed-by:` (mw-tb6gdr9) records the advisory claimant while a task is doing/blocked: written by `start` via the MW-K1 identity chain, released by `close`/`drop`/`reopen` — a coordination signal, never a lock (concurrency stays git's problem); parallel starts that merge into one history are lint's `double-claim`, a claim outside doing/blocked its `claim-stale` — both reported, never auto-resolved. IDs are `<alias>-<7-char base32 random>` (32^7 ≈ 34.4B combinations; minted at 4 chars before mw-1b09, owner-ruled 2026-08-06 — length is a minting rule only, never validated at parse, so old 4-char IDs stay legal and stores mix lengths freely), collision-checked at creation against local files; parallel clones share no state and CAN collide — `lint` detects post-merge duplicates, `--fix` re-slugs (MW-A4). Filename = `<id>-<slug>.md`; the slug is cosmetic and never load-bearing — the ID prefix is what by-ID lookup globs on. Storage may grow (history); read-time output is what's capped (MW-A5/K4). The self-contained, versioned, normative spec of all of the above is `FORMAT.md` (mw-dg5j1sv) — this section keeps the rationale; FORMAT.md is what a third-party reader implements from, and where FORMAT.md and the binary disagree, FORMAT.md wins.

## 3. Ingestion pipeline

`discover docs/meshwork/*.md → parse frontmatter + tail sections (strict serde model; unknown keys warn, MW-A6) → validate (schema, edge targets, cycles via DFS on needs+parent, anchors, attachment paths) → row projection`. Files that fail to parse are carried as `status: invalid` rows (ID recovered from the filename, error text attached) so they stay visible in every listing and in lint — never silently dropped (MW-I2). The `.cache/tasks.jsonl` projection is keyed on (file count, total bytes, max-mtime); at ≤1K tasks/repo a full reparse is <100ms (MW-C4), so the cache is an optimization, not a dependency — v1 ships without it and reserves the layout.

## 4. Tables (the SQL contract)

| table | columns (abridged) |
|---|---|
| `tasks` | `gid` (`repo#id`), `repo`, `id`, `title`, `status`, `category`, `verify`, `waived` (reason or NULL — makes MW-E2's "queryable" true), `seq`, `created`, `blocked_reason`, `claimed_by` (advisory claimant or NULL, mw-tb6gdr9), `github`, `path` |
| `edges` | `src_gid`, `dst_gid`, `kind` (`needs`\|`parent`\|`discovered-from`\|`relates`), `resolved` (bool) |
| `labels` | `gid`, `label` (exploded) |
| `comments` | `gid`, `ord` (file position), `date`, `author` (self-professed), `text`, `hash` (the FORMAT.md identity hash, mw-xvtf5jx) (MW-C1/K1) |
| `log` | `gid`, `ord` (file position), `date` (as written — stamp, date-only, or history's junk), `from_status`, `to_status` (NULL for free-text entries), `note` — `## log` through the §2 grammar (mw-3wnhhvp); unlocks blocked-duration/cycle-time/activity queries |
| `repos` | `repo`, `path`, `remote`, `present` (bool) |

Single-repo commands register the same tables filtered to one repo — one code path (MW-G3). Edge direction: `parent` edges are stored child→parent (`src` = the child). `edges.resolved` is derived at ingest (dst found in the loaded set or via registry lookup); the `ready` SQL's NULL-join is its equivalent — reports use the column, the normative SQL uses the join.

## 5. Canned verbs = frozen SQL

`ready` (normative, MW-B6):

```sql
SELECT t.id, t.title, t.claimed_by, t.verify FROM tasks t
WHERE t.status = 'open'
  AND NOT EXISTS (              -- unmet hard deps block (MW-B6)
    SELECT 1 FROM edges e
    LEFT JOIN tasks d ON e.dst_gid = d.gid
    WHERE e.src_gid = t.gid AND e.kind = 'needs'
      AND (d.status IS NULL OR d.status NOT IN ('done','dropped')))
  AND NOT EXISTS (              -- containers with live children aren't actionable (MW-B6/B8)
    SELECT 1 FROM edges c JOIN tasks ch ON c.src_gid = ch.gid
    WHERE c.dst_gid = t.gid AND c.kind = 'parent'
      AND ch.status IN ('open','doing','blocked'))
ORDER BY coalesce(t.seq, 999999), t.created
LIMIT 20;
```

`d.status IS NULL` = unresolved edge counts as blocking — conservative by rule (MW-G5). Single-repo commands do NOT leave cross-repo targets unresolved when they don't have to: a foreign `repo#id` resolves through the registry with a direct file lookup (`<repo-path>/docs/meshwork/<id>-*.md` — the ID-prefixed filename exists precisely for this), no full portfolio load; only an unregistered or absent repo yields NULL (MW-B3/G5). `why <id>` walks `needs` transitively and prints the frontier of actually-open blockers with their `blocked_reason`/verify. `tree <id>` walks `parent` downward — at any depth, so a saga→epic→story→task chain renders as a tree without the tool knowing what those words mean (MW-B8). `blocked` lists `status='blocked'` + reason. All verbs share the caps (20 rows; last 3 comments) with `… and N more` (MW-D2).

## 6. CLI surface (complete for v1 — anything not here is a non-goal)

| verb | does |
|---|---|
| `init` | create `docs/meshwork/` + config in a repo |
| `add "title" [--cat p] [--label l] [--needs id..] [--parent id] [--from id] [--verify cmd] [--seq n] [--docs link..] [--dry-run]` | create task file, print id; missing `--verify` = lint warning until set (MW-E2); `--dry-run` prints the would-be file, writes nothing (mw-0wvndqa); curated unknown verbs (log→comment, done→close, rm→drop) fail with a two-line did-you-mean — error-text only, not surface (mw-5hrb22q; the `--category`/`--doc` alias half awaits its own §6 nod, mw-42ygb52) |
| `add --batch <file\|-> [--dry-run]` | several tasks atomically (mw-af4kbjy): concatenated §2 documents, `id:` omitted, local-only `handle: <name>` — `@name` legal anywhere an id is (needs/parent/from/relates); ids minted, refs rewritten, all files or none; unknown frontmatter keys refuse the whole batch, and `from:` is an input alias rewritten to the canonical `discovered-from:` (mw-16pyc5g); `--dry-run` prints the would-be files, writes nothing |
| `set <id> [--seq n] [--docs link..] [--handoff "text"\|@file\|-] [--cat p] [--verify cmd] [--title "t"]` | field edits without opening the file (mw-0f4j); `--docs` appends, `--handoff` replaces the block — `@file`/`-` (stdin) spellings land prose without transiting shell quoting (mw-rz4ey2h, ruling 2026-08-10); `--cat`/`--verify`/`--title` grown by owner ruling 2026-08-10 (mw-f1x71yg — nine pilot sessions hand-rewrote files for these) — retitling never renames the file (the slug is cosmetic), and a replaced `verify:` re-arms the MW-E5 approval gate by construction |
| `show <id> [--docs] [--comments]` | full task; last-3 comments by default (MW-K4); `--docs` = anchor-scoped excerpts, capped ~4KB/link (bytes, MW-D5/F2); `commits:` tail derives the closing work from `git log --grep=<id>` (id-in-subject convention, local refs only, retroactive — mw-ntn0t32; close also anchors HEAD into the `→done` note) |
| `comment <id> [--as <author>] "text"\|@file\|-` | append comment; `--as` falls back to `$MESHWORK_AUTHOR`, then config `default_author`, else error (MW-K1); `@file`/`-` per mw-rz4ey2h |
| `attach <id> <path>` | copy file into `attachments/<id>/`, record in frontmatter; refuses overwrite without `--force` (MW-K2) |
| `start [--as <author>] / block --reason / drop / reopen <id>` | status transitions + log line; `start` refuses a task with no `verify:` — needs-verify; writing the done-test is the first unit of the work (mw-6wdpz1b, amending MW-E2) — and records an advisory `claimed-by:` when the MW-K1 chain resolves an identity (no identity = no claim, never an error); close/drop/reopen release the claim, block keeps it (mw-tb6gdr9); `reopen`: blocked\|doing\|done → open (the missing inverse — without it every unblock is a hand-edit) |
| `close <id> [--waive "reason"] [--approve]` | run `verify:`, close on exit 0 only (MW-E2); shell runs sit behind the MW-E5 trust gate — `--approve` shows the text and records this clone's approval, `MESHWORK_TRUST=1` is the reviewed-checkout grant (§12b, mw-9rc4vs6) |
| `dep add / dep rm <a> --needs <b>` | edge edits without opening the file |
| `ready / blocked / tree / why` | §5 |
| `q "SELECT …" [--json]` | raw SQL (MW-C1) |
| `prime` | §7 |
| `lint [--fix]` | schema, cycles, anchors, attachment size, post-merge damage (MW-I2/K3) |
| `mirror push / mirror status` | §8 |
| `portfolio ready / next / q / seq` | §9; union pipeline + sequence overlay |
| `import todo <path>` | §10 migration |

Every verb: `--json`, stable schema, versioned in-band — the envelope is `{"meshwork": {"version": "<crate>", "schema": 1}, "verb": …, "data": …}` (MW-C3 as amended by mw-5kp033j; `schema` is the former `v`).

## 7. Session integration (where the savings land)

`prime` emits, capped at 6KB ≈ 1.5K tokens (normative approximation: 4 bytes/token; the gate measures bytes — MW-D3/D5): ready top-10 (one line each) · in-progress with last log line · blocked with reasons · counts (`12 open, 3 blocked, 41 done`) · store provenance (`store @ 3f5ff64 · 2 uncommitted task edits · 1 ahead of upstream`, scoped to docs/meshwork/, local refs only; any git failure omits the line silently — mw-3jwwh5d). Injected via SessionStart hook (the giles `session_context.py` pattern) or run as the session's first command per CLAUDE.md ritual. Task selection then goes `prime → show <id> [--docs]` — flat two-level disclosure (MW-D1), replacing the ~22K-token full-file read measured in ~95% of sessions. Session end: `close`/`block`/`comment` on touched tasks only, plus refresh `handoff:` on whatever is up next. There is no HANDOFF.md — prime is the handoff (§7b); durable state lives in task logs, comments, and §15.

### 7b. prime as materialized handoff (owner-ruled 2026-08-06, four rounds; lands with mw-a8tv)

Hand-written HANDOFF.md is retired: it duplicates graph state. prime becomes the full handoff view, same 6KB cap, sections in order:

1. **headline** — counts + category rollup capped at top 5 groups (group by first two category segments; rank by min seq among open members — seq is the priority primitive, there is no priority field; rest collapses to `… +N`, MW-D2 pattern).
2. **weather** — all derived, never stored: freshest comments across the active frontier (ready+doing+blocked, newest first, byte-capped) + blocked-with-reasons.
3. **next** — top ready task: its `handoff:` commentary FIRST, then category, blocks-line (what it unblocks), verify, docs: refs, body head verbatim, last-2 comment tail (MW-K4).
4. **also-ready** one-liners with blocks-lines.
5. **recently done** — last ~5 closed (id, title, done-date from log lines).

New frontmatter key `handoff:` (multi-line block): the outgoing session's color commentary to the incoming one — the ONLY authored piece of the view, meaningful solely on up-next tasks. Rewritten freely (history belongs in comments); set via `meshwork set <id> --handoff` or hand-edit — both legal (mw-0f4j superseded the original hand-edit-only ruling, owner 2026-08-06). Lint warns when `handoff:` sits on a done task. Session-end ritual: refresh `handoff:` on whatever is up next; comment anything history-worthy. Landing commit: delete docs/HANDOFF.md; drop CLAUDE.md ritual step 4's HANDOFF clause + baseline override 2; adoption-skill step 4 becomes "delete HANDOFF.md"; move HANDOFF's decisions line to §15 and its 2.3 re-bless note into mw-k7r5's body.

## 8. GitHub push (append-only view, MW-H*)

Per-repo opt-in (`config.toml`: `mirror = true` or a `[mirror]` table), via the `gh` CLI (house auth), dry-run by default with `--yes`.

**Branch guard (mw-pvfrpd4).** The store rides branches like code, but the mirror is append-only and unretractable — a push from a feature branch publishes issues/comments for state that may rebase away or never merge. `mirror push` therefore refuses off the repo's default branch, naming both branches, exit nonzero. The default branch is the local `origin/HEAD` ref (zero network, MW-J6); when it's unset the default is indeterminate and the push refuses too, naming the fix (`git remote set-head origin <branch>`). `[mirror] allow_non_default = true` skips the guard but announces itself in output every time — loud, never silent. The guard runs before any M3 push logic and its contract precedes that implementation.

**At creation (once):** search the repo's issues for the `<!-- meshwork:t:<id> -->` marker first — a hit means another clone already created it: adopt that number into frontmatter instead of duplicating. Then create: title, body (description + the task-ID marker + backlink to the task file path), labels incl. `cat:engine/spill`; `needs`→blocked-by and `parent`→sub-issue where gh/GraphQL supports relationship *creation* (additive); if the installed gh can't, skip with a warning — relationships are cosmetic on the mirror.
**Afterwards (append-only):** task comments push as issue comments prefixed with their self-professed author (the GitHub author is the token owner, so the claimed identity rides in the comment text); status transitions and local title/body edits surface as comments (`meshwork: doing → done (verified exit 0)`), never as edits; attachments are linked by their blob URL on the remote at the pushed commit — the file is already in git, so nothing is uploaded out-of-band.
**Never:** close, reopen, delete, or edit state/title/body/labels post-creation (MW-H2). A locally-dropped task posts a comment; the open issue is a human's to close.
**Idempotency (MW-H3):** `github:` issue number in frontmatter (set once, merge-safe). Each pushed comment ends with `<!-- meshwork:c:<hash8> -->` (the first 8 hex chars of the FORMAT.md comment identity hash — SHA-256 over date NUL author NUL text, mw-xvtf5jx); `mirror push` lists the issue's comments, collects markers, and posts only unmarked local comments. No pushed-state counters in frontmatter — a counter merges wrong the moment two clones interleave comments. A second `mirror push` with no local changes is a no-op (acceptance gate, REQUIREMENTS §4). `mirror status` diffs local vs remote and prints a drift report for a human — never writes locally (MW-H4). giles' lesson applied: keep the *reconcile-as-report* half, drop the write-back half entirely.

## 9. Portfolio & master sequencing

`repos.toml` names each repo (name = cross-repo edge namespace, MW-B3/G2). `sequence.md` is a human-editable ordered list of `repo#id` refs under optional tranche headings; `portfolio next` = first sequenced ready task, then falls back to per-repo `seq`. Resequencing = editing one small file in one small repo — reviewable, diffable, backed up (unlike the code root today).

Registry durability (mw-mrjccx2): a repo entry may carry `aliases = ["oldname"]` — refs baked into other repos' files survive a rename because resolution accepts former names, while registry-aware lint warns `renamed-repo` with the exact rewrite (never silent, never auto-fixed). Namespace damage — one name claimed by two entries across names+aliases — is a `registry-collision` error, and two locally-present registered repos minting the same ID alias prefix (their stores' `config.toml alias`) is an `alias-collision` error: bare-ID lookup is ambiguous the moment it happens, so it never happens quietly. Absent/unreadable repos are skipped by the alias check, never guessed (MW-G5's spirit). Portfolio-dir discovery (mw-9093): `MESHWORK_PORTFOLIO=<dir>` overrides; the default is `~/Documents/code/portfolio` (§15.4). `portfolio` verbs discover through that chain and load every registered store — the union is a *loading* concern feeding the same `session_for` pipeline single-repo verbs use (one code path, MW-G3); a registered repo that can't load skips with a report (`no-path`/`no-checkout`/`no-store` — stderr in text mode, a `skipped` list in JSON data), but a present-yet-broken store is a loud error: its tasks silently missing would misreport the portfolio. Single-repo verbs use the same quiet chain for *dep resolution* (mw-k7r5, §5): cross-repo `needs` targets resolve by direct file lookup, and only TERMINAL statuses inject task rows — done/dropped satisfying a dep is the one delta the frozen predicate needs, an injected open task would leak into listings, and NULL already blocks conservatively; no registry anywhere silently means today's conservative behavior, while a found-but-broken registry stays a loud error. Single-repo `lint` findings remain env-opt-in — registry *hygiene* reporting is portfolio work; resolution is repo work (MW-G1's scope holds either way). Local checkout paths come from the gitignored `repos.local.toml` `[paths]` table, defaulting to `~/Documents/code/<name>`. Override semantics (mw-5ckb): absolute values pass through; `~/` expands against HOME and is loud when it can't resolve; relative values anchor at the portfolio dir (the only deterministic anchor for a per-machine file). Keys share the name+alias namespace — a former-name key applies but warns `renamed-repo` with the rewrite; a key matching no entry warns `unknown-path-override` (the file is gitignored, a typo has no other review surface); two keys overriding one entry is an `override-collision` error. An absent local file is the normal state; a present-but-broken one is a loud error, never a silent skip.

## 10. Migration (one session per repo, MW-J3)

`import todo TODO.md`: parses baseline checkbox format (`[ ]`/`[~]`/`[x]`/`[!]`, `verify:` lines, `## Now` ordering → `seq`) into task files; prose pointers ("this IS ask #8") become real edges by hand during review. Old TODO.md → `docs/archive/` (history never deleted). `check-todo.sh` retired; `check-file-length.sh` gains a byte gate for surviving docs (closes the 38KB/131-line hole). Baseline amendment lands only after the sazed pilot passes REQUIREMENTS §4.

## 11. Phasing with stop-lines

| phase | contents | stop-line |
|---|---|---|
| M0 | store, parse/lint, `add/show/start/close`, `ready`, `q` — single repo | usable in sazed pilot |
| M1 | full edges, `tree/why/blocked`, categories/labels/levels, `comment`/`attach`, `prime` + hook, `import todo` | session ritual switched |
| M2 | portfolio union, `repos.toml`, `sequence.md`, cross-repo edges | leras joins |
| M3 | `mirror push/status` (append-only) | OEM-face visibility |
| M4 | `--docs` drill-through excerpts | — |

Gate-first inside M0: the test harness, fixture-corpus skeleton, and `verify_meshwork.sh` land before the first feature (§13–14). The ordered work items with per-item verify commands live in `PLAN-meshwork-build.md` — this table is the shape; that doc is the work.

**There is no M5.** Post-M4 work is bugfixes or requires an owner ruling amending REQUIREMENTS §3. Estimated M0–M1 ≈ 3–4 sessions given sahjhan's query module as a template (the extra session is the corpus, and it's not optional).

## 12. Risks

DataFusion compile time (house-accepted via sahjhan). YAML hand-edit footguns (strict schema + `lint --fix`). `serde_yaml` is archived — MW-J1 rules it out; decided: `serde_yaml_ng`, fallback `saphyr` (REQUIREMENTS J1). Union-merge's duplicate-key failure mode is accepted and lint-fenced (MW-I1/I2). Attachment bloat in git history (MW-K3 lint + excerpt-first culture). Name: **meshwork** verified unused across crates.io/npm/GitHub tooling 2026-08-04 (predecessor "taskmesh" was squatted; renamed same day). Biggest risk is the identified pattern: this is a new project in a portfolio whose ruled priorities (THREE-PATH 2026-08-02) don't include it — it stays unscheduled until the owner slots it, and REQUIREMENTS §3 is the fence that keeps it a tool, not a product.

### 12b. Trust boundary: `verify:` is untrusted input (ruled via mw-mjwfvxn, 2026-08-07)

**Threat.** Task files are data that arrive through git merge — a PR, a fetch, a synced clone. Anyone who can land a file in the repo controls every field in it, and `verify:` is the one field the tool ever *executes* (`sh -c` from the repo root, at `close`). The drive-by path: a merged task carries `verify: curl … | sh`; a later operator — human, or an agent closing tasks in a loop — runs `meshwork close <id>`; arbitrary code executes with that operator's credentials. The append-only mirror, agent-driven sessions, and portfolio-wide adoption all widen exposure: `close` runs in more places, read by fewer eyes.

**Execution points.** Exactly one today: `close` (MW-E2). `mirror` invokes `gh` with fixed argv — task content travels as arguments and stdin, never through a shell — and stays that way by rule. Adjacent surfaces named, not covered here: repo-relative path resolution (`docs:`, attachments) and terminal escape sequences in rendered content are injection surfaces of a different class — file them as their own tasks; this ruling is about code execution.

**The boundary (ruled).** Trust attaches to the *exact verify text, per task, approved by the operator of this clone* — trust-on-first-use, the direnv-allow pattern. Approvals are content hashes in the gitignored `.cache/` (mw-9rc4vs6): per-clone local state that can never arrive via merge, exactly because merged content is what's untrusted. Git metadata is NEVER a trust signal — author and committer are self-professed strings (same rule as MW-K1 identity), trivially spoofable by the attacker this section is about. A fresh clone trusts nothing until its operator does.

**CI/test posture (ruled).** `MESHWORK_TRUST=1` joins the §15.6 env contract: it asserts "the operator vouches for every `verify:` in this checkout" — for CI, the gate, and test harnesses, where the checkout itself was reviewed before the runner touched it. It is a per-invocation, deliberate grant; nothing in the tool sets it, and it never belongs in a login shell.

**CLI surface delta (ruled).** No new verb. The trust gate may add exactly one flag to §6: `close --approve` (print the verify text, record the approval, run). Refusal without approval is loud and names the approval step; approval requires the text on screen.

**Scope (ruled).** A constrained declarative verify grammar (mw-sascrgs — parse, don't shell) is in scope: REQUIREMENTS §3's "no bespoke query language" fence bans *query* DSLs — SQL stays the only query surface — and was never about the execution side. Sandboxing actual shell (containers, seccomp) stays out of scope: the TOFU gate closes the drive-by path, and reviewed-then-approved shell is the operator's own risk, as it is for any script in the repo.

## 13. Test architecture & fixture corpus (MW-J4/J6)

Three tiers, all offline, all inside the gate.

**Unit** — parser, strict schema, ID generation, cycle DFS, segment-prefix matching, byte-budget truncation. Plain `#[test]`, no fixtures.

**Integration: the committed fixture corpus** — under `fixtures/` in the meshwork repo, copied to a tempdir per test (tests never mutate the corpus):

```
fixtures/
  alpha/meshwork/          # "az": the kitchen-sink repo — ~30 tasks: a 5-deep
                           # saga→epic→sprint→story→task chain (B8), every status,
                           # every edge kind, cross-repo needs → beta, needs → gamma
                           # (absent repo), docs: links w/ good+bad anchors, multi-author
                           # comments w/ continuation lines, attachments (one >1MB for
                           # the K3 warning), seq gaps, one task with no verify:,
                           # log lines in every §2 grammar shape: transition w/ and w/o
                           # note, free text, date-only and minute-res stamps (mw-3wnhhvp)
  alpha-broken/meshwork/   # lint corpus: needs-cycle, parent-cycle, cross-repo parent,
                           # blocked w/o reason, duplicate-ID pair, duplicate frontmatter
                           # key (post-union file), unparseable YAML, dangling edge,
                           # unknown field
  beta/meshwork/           # "bz": small clean repo; target of cross-repo edges
  portfolio/               # repos.toml (alpha, beta, gamma-absent) + sequence.md with
                           # tranches; harness writes repos.local.toml w/ tempdir paths
  golden/                  # committed expected outputs, byte-compared:
                           # ready-alpha.json · prime-alpha.txt · why/tree/blocked-*.json
                           # portfolio-ready.json · portfolio-next.txt · lint-broken.json
                           # show-docs-*.txt · mirror-push-1.calls · mirror-push-2.calls
```

Goldens regenerate only via `--bless` + a reviewed git diff (MW-J4) — never silently.

**E2E scenario tests** — each drives the real binary and real git in a tempdir:

1. *concurrent-worktrees*: clone alpha twice; A creates + closes tasks, B creates + comments on the same task A commented on; merge → zero conflict markers (union attr), lint clean, both comments present (MW-I1; gate scenario from REQUIREMENTS §4).
2. *duplicate-id-merge*: seeded RNG forces the same ID in both clones; merge; `lint` reports, `lint --fix` re-slugs and rewrites edges, lint clean (MW-A4).
3. *union-poison*: both clones set the same task's status differently; merge; strict parse rejects the duplicate key; the row surfaces as `invalid` in listings; `lint --fix` repairs (MW-I2).
4. *mirror-idempotent*: stub `gh` on `$PATH` records calls, replays canned responses; push twice → `mirror-push-2.calls` golden is empty; pre-seeded issue with the task marker → adopt, not duplicate (MW-H3/J6).
5. *offline-everything*: `$PATH` without `gh`, no network — every non-mirror verb runs clean (MW-H5).
6. *absent-repo*: gamma unregistered → its edges unresolved, reported, conservatively blocking, exit 0 (MW-G5).
7. *migration*: `import todo` on a committed sample in sazed's real TODO.md format → golden task set (MW-J3).

Every MW-* MUST maps to ≥1 named test in `TRACE.md` (seeded in PLAN §3); the gate checks the map, so an untested requirement is a gate failure, not a review vibe (MW-J5).

## 14. Gate — `verify_meshwork.sh` (MW-J5)

House pattern (verify_alpha.sh precedent: numbered sections, one exit 0):

| § | check | fails when |
|---|---|---|
| 1 | `cargo fmt --check` | drift |
| 2 | `cargo clippy --all-targets -- -D warnings` | any warning |
| 3 | `cargo test` — unit + integration + e2e, offline, stub gh | any failure |
| 4 | `cargo llvm-cov --fail-under-lines 80` | coverage <80% (house number) |
| 5 | file caps: warn >500, fail >750 lines per source file (house numbers) | ceiling breach |
| 6 | trace: every MW-* MUST in REQUIREMENTS appears in TRACE.md mapped to a test name that exists in the test binaries | unmapped requirement or phantom test |
| 7 | perf (owned machines, release build): `ready` cold <100ms at 1K synthetic tasks, portfolio <1s at 20 synthetic repos; N≥7 reps, median (MW-C4) | regression |
| 8 | self-host (from M1): `meshwork lint` + `meshwork prime` clean on meshwork's own `docs/meshwork/`; prime output ≤6KB measured | dogfood breakage |

No network anywhere in the gate (MW-J6). The live scratch-GitHub drill (REQUIREMENTS §4) is a manual acceptance step outside it.

## 15. Decisions (formerly open questions — resolved 2026-08-04 so "go" means build, not deliberate)

1. **Single category per task.** Cross-cutting axes are what labels are for; prefix queries stay trivial. Revisit only with a concrete query it can't express.
2. **`seq` = integers with gaps of 10** (10, 20, 30…); inserts take midpoints; `portfolio seq`/repo-level renumber rewrites cleanly when a gap exhausts. No fractional/lexicographic cleverness.
3. **`--waive` stays available everywhere**, recorded and loud (`WHERE waived IS NOT NULL` — the `waived` column exists for this). Strict mode is a non-goal until the pilot shows waive abuse actually happening.
4. **Portfolio repo = `~/Documents/code/portfolio/`**, its own git repo with a private GitHub remote as backup. Folding into a future backed-up code-root repo is a later `git mv` + one config path, not a blocker.
5. **Comment growth: accepted.** Storage is history; read-time is capped (MW-A5/K4). Lint warns when a task file passes 64KB — the real signal is usually that the task should be split.
6. **Determinism hooks are env vars**: `MESHWORK_ID_SEED` (id generation), `MESHWORK_TODAY` (clock), `MESHWORK_BLESS` (golden re-bless). Tests and fixtures depend on them; they are contract, not convenience. (Moved from hand-written HANDOFF at its retirement, 2026-08-06.) Later additions to the contract: `MESHWORK_TRUST` (MW-E5 reviewed-checkout grant, §12b) and `MESHWORK_PORTFOLIO` (portfolio-dir override — the only registry trigger for single-repo lint, and the test/nonstandard-machine override for `portfolio` verbs, whose default is `~/Documents/code/portfolio` per §15.4; §9 — mw-mrjccx2, mw-9093).
7. **No hand-written handoff docs** (2026-08-06, four review rounds): prime IS the handoff (§7b). Current conditions are always derived; the only authored voice is `handoff:` on up-next tasks. Per-commit messages carry rationale; durable decisions land here in §15.
8. **Minted ID suffix is 7 chars** (owner-ruled 2026-08-06, mw-1b09; was 4). 32^7 ≈ 34.4B combinations retires the parallel-clone collision worry for any realistic store. Length is a minting rule, not a validation rule: parse accepts any suffix, pre-ruling 4-char IDs are legal forever, and no store migration ever happens for this.
9. **Store root is `docs/meshwork/`, flat** (owner-ruled 2026-08-06, mw-acgp; was `meshwork/tasks/`). Task files sit beside config.toml — only `.md` files are tasks, so no `tasks/` level earns its path segment. The union attribute anchors to the store dir (`/*.md`). Pre-move stores migrate with a `git mv`, nothing else — the tool never migrates layouts itself.
10. **Terminal tasks auto-archive to `archive/`** (owner-ruled 2026-08-06, mw-45e2qf4). close/drop move the file, reopen moves it back, import routes already-terminal items there, `lint --fix` sweeps strays (`misplaced` warning). Owner-confirmed constraint: archive is ALWAYS loaded — tables, needs-resolution, and prime are location-blind; only the directory tidies. IDs stay collision-checked across root+archive (MW-A4 never-reused). No new verb — automagic rides the existing lifecycle.
11. **`verify:` is untrusted input** (ruled 2026-08-07, mw-mjwfvxn): TOFU per-clone approval before any shell verify runs; `MESHWORK_TRUST=1` is the reviewed-checkout escape hatch; git authorship is never trust. Substance in §12b; enforcement lands with mw-9rc4vs6, grammar with mw-sascrgs.
