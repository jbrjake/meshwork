# FORMAT.md — the meshwork on-disk format, version 2

Normative and self-contained: a third party can implement a reader from this file without the binary. Rationale lives in docs/DESIGN-meshwork.md; requirement IDs (`MW-*`) in docs/REQUIREMENTS-meshwork.md. Where this file and the binary disagree, this file wins and the binary has a bug. A conformance corpus — a golden store plus its expected projection, self-checkable by any reader — lives at `fixtures/conformance/` in the meshwork repo (mw-7c6svyn).

**Versioning.** The store declares its format in `config.toml` (`format = 2` is current; absent means 1). The version bumps only on a *semantic* change — one that would make an old reader misread existing bytes. Additive change ships without a bump under the minting-rule idiom: new writers may mint richer forms (longer IDs, minute stamps, new log shapes), but parsers accept the old forms forever and never validate mint-time rules. A reader encountering a format newer than it knows MUST refuse loudly, never guess. The store, not the file, is the versioning unit (mw-1bb2542): `format` covers every file in the store, and a bare task file encountered outside one — pasted into an issue, emailed — carries no version and is read at the reader's current format version. Machine output carries the same number: the binary's `--json` verbs wrap results in an envelope, `{"meshwork": {"version": <binary version>, "schema": <n>}, "verb": …, "data": …}`, and the envelope's `schema` IS this format version — one contract, one number (mw-5rgq9ka). `version` identifies the producing binary and moves per release; `schema` moves only when `format` does. Format 2 adds archive bundles (*Store layout*): a format-1 reader would take a bundle for one task, hence the bump. Every format-1 store reads unchanged under a format-2 reader, and a writer declares 2 only when it creates the store or writes its first bundle (mw-bvxpeef).

## Store layout

```
docs/meshwork/               # the store root, flat
  config.toml                # presence marks a store; see below
  .gitattributes             # "/*.md merge=union\n/archive/*.md merge=union\n"
  .cache/                    # reserved scratch, .gitignore'd ("*\n!.gitignore\n");
                             #   deletable at any time, never a dependency
  <id>-<slug>.md             # one live task per file
  archive/<id>-<slug>.md     # terminal tasks (done|dropped); same format, always loaded
  archive/bundle-NNNN.md     # format 2: many terminal task documents, concatenated
  attachments/<id>/<file>    # attachment payloads, plain files
```

Only `.md` files directly in the store root and in `archive/` are task files. Terminal tasks live in `archive/`; location carries no semantics beyond tidiness — every reader MUST load both directories identically. Nothing outside `docs/meshwork/` belongs to the store, and the store never references files outside its repo.

**Bundles (format 2).** In `archive/`, a file named `bundle-<digits>.md` is a bundle: a sequence of task documents, not a task. A document opens at a top-level `---` line directly followed by an `id:` line; its frontmatter runs to the next `---` line; its body runs to the next opener. A `---` inside a fenced code block, or one not followed by an `id:` line (a horizontal rule), is body. Documents are separated by one blank line. A bundled task takes its id from its `id:` line — the filename carries none — and a reader MUST load every document exactly as it loads a single file, the bundle's path standing as the task's path. A bundle holds only terminal tasks (a live one there is the reference binary's `misplaced` finding). Writers append whole documents; a document leaves a bundle only by being split back out as a single file (`reopen`). The reference binary bundles nothing until 100 archived files are loose, then folds them under `lint --fix`, 512 KiB per bundle (DESIGN §1).

## config.toml

TOML; unknown keys are ignored (config is not the strict surface task files are).

| key | meaning |
|---|---|
| `alias` | string, required, `[a-z0-9]+` — the ID prefix for tasks minted in this store; a dash or uppercase would corrupt filename ID recovery (first two dash-segments of the stem) |
| `format` | integer — format version; absent = 1; 2 means `archive/bundle-NNNN.md` files may exist (see *Bundles*) |
| `default_author` | string — fallback identity for comments/claims |
| `[hierarchy] levels` | string list — display names for category depths; zero semantics |
| `mirror` | bool — GitHub mirror opt-in; absent = off |
| `[stats] window_days` | integer — the width of the derived projection's `_w` window (see *Views*); absent = 7 |

## Task file

Filename: `<id>-<slug>.md`. The slug is cosmetic and never load-bearing; the ID prefix is what by-ID lookup globs on (`<id>-*.md`). A file is: YAML frontmatter between `---` fences, then a free markdown description, then optional tail sections `## log` and `## comments` (in that order), each holding `- ` bullet entries whose continuation lines are indented two spaces.

**Strictness.** Unknown frontmatter keys warn, never fail. Duplicate top-level keys (union-merge damage), missing fences, YAML errors, or schema violations make the file *invalid*: readers MUST surface it as a loud row (ID recovered from the filename — the first two dash-segments of the stem) in every listing, and MUST NOT silently drop it.

**Frontmatter schema** (all keys except `id`, `title`, `status` optional):

| key | type | meaning |
|---|---|---|
| `id` | string | `<alias>-<suffix>`; suffix minted as 7 chars of lowercase Crockford base32 (`0123456789abcdefghjkmnpqrstvwxyz`); length/alphabet are minting rules — parsers accept any suffix |
| `title` | string | one line |
| `status` | enum | `open` \| `doing` \| `blocked` \| `done` \| `dropped` |
| `category` | string | one slash-path, arbitrary depth (`engine/spill/budget`) |
| `labels` | string list | flat, orthogonal to category |
| `needs` | id list | hard deps; `repo#id` crosses repos |
| `parent` | id | same-repo nesting, child points up |
| `discovered-from` | id | provenance edge |
| `relates` | id list | soft links |
| `to` | string | addressee of an ask: a repo name or `repo#id` — see *Addressed tasks* below |
| `answers` | id | the ask gid this task answers; projects as an `answers` edge, never gates readiness |
| `verify` | string | close-gate command — DSL predicates (preferred, DESIGN §12b) or legacy shell; untrusted input (MW-E5); readers treat it as opaque text and never execute it |
| `docs` | list | repo-relative paths, optional `#§-anchor`; or `repo#path[#§-anchor]` for a doc in a registered sibling repo — the `needs:` spelling, resolved through the portfolio registry and confined to that repo, never a `../` path (mw-8q0srvb). A head with no `/` and no `.` is a repo name when the registry says so and a local path when that file exists; an unregistered name is reported, never read |
| `attachments` | list | store-relative `attachments/<id>/<file>` paths |
| `seq` | integer | per-repo order weight; lower = sooner; gaps of 10 by convention |
| `github` | integer | mirror issue number; set once, never changes |
| `created` | string | date stamp as minted |
| `blocked-reason` | string | required non-empty when `status: blocked`; a stale reason left behind on an unblocked task is legal — at most a lint finding, never invalid |
| `claimed-by` | string | advisory claimant while doing/blocked; a claim, never a lock |
| `waived` | string | reason recorded by `close --waive` |
| `handoff` | string | authored note to the next session — inert to the format: carried and projected, never interpreted; which task is "up next" is a consumer's judgment (the reference binary's `prime`), never this spec's (mw-4jgrjar) |

**Stamps.** Minted stamps are UTC minute resolution: `YYYY-MM-DDTHH:MMZ` (17 chars). Date-only `YYYY-MM-DD` is legal forever. These two are the only conforming forms; anything else carrying a date prefix — an offset stamp like `2026-08-06T21:47-04:00`, a seconds field, a space separator — is nonconforming (mw-8x954nr): never minted, compared as opaque text exactly as written, and a reader MAY warn. Stamps sort lexicographically, and the ordering guarantees hold only among conforming forms — in particular, date-only is a strict prefix of every minute stamp of its own day, so date-only sorts before them and max-stamp prefers the more precise entry. A nonconforming stamp still participates as written: a same-minute offset form sorts before its `Z` twin and its civil time is never computed — the cost of minting one. Last-activity of a file is the max stamp in it, always derived, never stored.

**Identity strings** (comment authors, `claimed-by`) are self-professed free strings — no accounts, no verification; an identity is a claim, recorded as claimed.

**Block values.** A key's value may span lines as a YAML block — a block scalar (`handoff: |`) or a block list (`docs:` / `needs:` items) — and such a block may carry unindented blank lines between its lines, which is legal YAML and the shape hand-authored paragraphs take. The block is one value: a writer replacing or removing the key MUST take every line of it, blank lines included, and never leave indented lines stranded under the next key (mw-87gxe7q). Indented lines left stranded after a blank under a plain `key: value` are exactly that damage; the reference binary's `lint --fix` drops them and logs the repair.

**Addressed tasks.** An *ask* is a task carrying `to:` — it lives in its author's store and is never copied anywhere. Delivery is a read-time join: a reader assembling repo R's view SHOULD surface every non-terminal ask in the loaded set whose `to:` repo-segment (the text before `#`, or the whole value) equals R, until a `done` task anywhere in the set carries `answers:` naming the ask's gid. A non-`dropped` answer that is not yet `done` is an intent, not an answer: the ask stays surfaced and the reader SHOULD render the answering task beside it as `answered-by <gid> (<status>)`, choosing a `done` answer over a live one and then the smallest gid — the `asks` view's `answer_gid`/`answer_status`. A `dropped` answer is no answer at all (MW-L2). There is no transport, no broker, and no write into the addressee's store; an unavailable sender repo simply contributes nothing. (mw-hfvtx0s) A surfaced ask SHOULD carry its age — whole days from `created` to the reader's clock, or nothing when `created` is absent — so an unanswered ask reads as a wait, not a row (mw-r6g9bhe). On the sending side a non-terminal task carrying `to:` is owed by its addressee, not worked in its own store: a reader listing a store's actionable work SHOULD leave it out and list it apart, with the addressee, the age and the same `answered-by` state (MW-L5).

## Tail-section grammars

**`## log`** — append-only, one entry per line:

```
entry      = "- " transition | "- " freetext
transition = date " " status "→" status [" — " note]
freetext   = date [" " text]
date       = first whitespace-delimited token, as written
status     = "open"|"doing"|"blocked"|"done"|"dropped"
```

Parsing is positional and never validates history: token one is the date as written; a second token reading `<status>→<status>` makes the entry a transition (note = the rest, with one leading `— ` stripped if present); anything else is free text. Token one is the date even when it isn't date-shaped — a hand-written `- fixed the thing` projects `date='fixed'`, and a consumer wanting real dates filters by the conforming stamp shapes above; the date is absent only when the entry has no text at all (a bare `- `) (mw-e60thg2). Writers mint the em-dash separator; parsers accept its absence. Minted free-text forms include `created`, `close attempt — verify exit <N>`, and `handoff by <author>` (`handoff` alone when no identity resolves) — written by `set --handoff` beside the block it replaces, so a handoff carries an author and an age (MW-S10; the `events` view parses the actor from it); minted transition notes include a block reason, `claimed by <author>`, `verify exit 0`, and `waived: <reason>` — and `→done` notes may end with an ` @ <short-sha>[+N]` anchor (repo HEAD when the close ran, `+N` uncommitted paths; mw-ntn0t32). All of it is note text under the grammar above, and extracting the anchor stays optional — but its form is normative for any consumer that does (mw-1byhnj1): a `→done` note carries an anchor iff it matches ` @ ([0-9a-f]{4,40})(?:\+([1-9][0-9]*))?$` — group 1 the short sha exactly as git minted it (lowercase hex, ≥ 4 chars), group 2 the uncommitted-path count, present only when the tree was dirty (a clean tree omits it; `+0` is never minted). A note that doesn't match simply has no anchor — absence, never an error. One agreed pattern instead of one regex per consumer; "which tree did the verify pass against" is the audit field of verify-gated closure and must not fork.

**`## comments`** — append-only, one entry per line:

```
entry = "- " date " [" author "] " text
```

Neither date nor author may be empty; a nonconforming entry is a warning, and the line is skipped, not fatal. A comment's identity is `SHA-256(date NUL author NUL text)` as lowercase hex — date and author as written, text with continuations joined by `\n`, `NUL` a single zero byte (mw-xvtf5jx). Every consumer that dedups comments (the mirror, UI layers, replication) MUST use this hash; the mirror's issue-comment markers abbreviate it to the first 8 hex chars. Minute-resolution stamps are what make it trustworthy: same-day identical text no longer collides.

**Stray content.** Inside a tail section, only `- ` bullets, two-space continuations, and blank lines belong to the grammar; a `## ` heading other than the tail headings switches to an ignored block that runs until the next tail heading. Readers MUST ignore all such stray lines (warn, don't fail) — but ignored is invisible to every projection, and in practice the lines are `cat >>` damage: body text the author meant to keep. `lint` flags stray tail content on live tasks and `lint --fix` relocates it above the first tail heading, order preserved, never dropping or inventing a line (mw-t01ek6s, mw-n3xgfs0).

**External evidence references** — a reserved convention, not a parse rule (like the `@ <short-sha>` anchor above: optional to extract, normative in form once you do). A log note or comment MAY cite an event in an external append-only evidence ledger as `[evt:<ledger>:<hash8>]` — `<ledger>` a lowercase slug naming the ledger, `<hash8>` the first 8 lowercase hex chars of the event's identity hash in that ledger's own terms. The reverse convention is the same key: an external event attesting to a task SHOULD carry its `gid` verbatim. Any attestation tool can then join events to task history with plain SQL — an agreed key and nothing more; no integration, no execution, no network (REQUIREMENTS §3; mw-cp11qh0).

## Merge semantics

The store is safe under concurrent edits in separate clones: creation is file-per-task; status edits touch one frontmatter line; log/comments append at end-of-file under the committed `merge=union` attribute.

The attribute is normative, not furniture: `docs/meshwork/.gitattributes` MUST carry `/*.md merge=union` and `/archive/*.md merge=union` (extra attributes on those lines are fine). Writers MUST ensure it exists; `lint` MUST error when either line is missing — a store without them keeps working and silently loses every guarantee in this section until the first concurrent edit — and `lint --fix` restores the missing lines (mw-mtn4hp8). Union's known failure mode — both sides editing the same frontmatter line — produces duplicate YAML keys, which strict parsing rejects into a loud invalid row; repair is mechanical (`lint --fix`): duplicate keys keep their first occurrence, except a duplicated `status:`, whose repaired value MUST be derived by replaying the `## log` — the latest transition wins (date order, later entry on ties) — because keeping either side's line by position silently loses the other side's transition (mw-efmgn6b). Duplicate IDs minted by parallel clones are detected post-merge and re-slugged. No locks, no daemon, no merge driver beyond the git built-in.

## Projection

The file→row projection is **stable and deterministic**: the store at commit X projects to exactly one set of rows, independent of platform, locale, load order, or wall clock. Each commit touching `docs/meshwork/` is therefore a well-defined delta of the task graph, and **git history is the change stream** — there is no other journal, and no verb will ever wrap one. Per-task history is the filename-prefix idiom:

```
git log -- "docs/meshwork/<id>-*" "docs/meshwork/archive/<id>-*"
```

Any external reader, UI layer, or incremental engine builds on (projection at X) + (commits after X). `.cache/tasks.jsonl` is reserved as an optional materialization of this projection; it is never authoritative and deleting it is always safe. Its freshness key, decided before any implementation exists (mw-n0r5jwm): a hash of the store's **content** — the projection inputs as sorted (relative path, bytes) pairs, each field length-prefixed — never mtime, size/count heuristics, or inode metadata. `git checkout` rewrites mtimes wholesale on every branch switch, so a metadata key thrashes in exactly the worktree-heavy workflow this format targets; the projection is a pure function of content, so content is the key. Same rule for any third-party cache built over this projection.

The projection is six tables. `repo` is the registry name from the portfolio's `repos.toml`, defaulting to the repo directory's name; `gid` is `repo#id` and is unique across a loaded set:

| table | columns |
|---|---|
| `tasks` | `gid` (`repo#id`), `repo`, `id`, `title`, `status` (the five values, or `invalid`), `category`, `verify`, `waived`, `seq`, `created`, `blocked_reason`, `claimed_by`, `github`, `addressed_to` (the `to:` key), `path`, `error` (invalid rows only), `body`, `handoff` (see below), `parent` (the `parent:` key as written; NULL when absent — a convenience projection of the one child-points-up edge kind, the `edges` row stays normative — mw-0ssk8dg) |
| `edges` | `src_gid`, `dst_gid`, `kind` (`needs`\|`parent`\|`discovered-from`\|`relates`\|`answers`), `resolved` (dst present in the loaded/registered set); `parent` edges stored child→parent; bare targets qualify with the declaring repo |
| `labels` | `gid`, `label` (exploded) |
| `comments` | `gid`, `ord` (1-based file position), `date`, `author`, `text`, `hash` (the identity hash above) |
| `log` | `gid`, `ord` (1-based file position), `date` (token one as written, even when it isn't date-shaped — mw-e60thg2; NULL only for an entry with no text at all), `from_status`, `to_status` (NULL for free text), `note` — the `## log` grammar above, exactly |
| `repos` | `repo`, `path`, `remote`, `present` |

**`tasks.body`** and **`tasks.handoff`** carry the free text. `body` is the description section: every line between the frontmatter's closing fence and the first tail-section heading (`## log` / `## comments` at top level — a heading inside a fenced code block is content, per the tail-section grammar above), with leading and trailing whitespace trimmed. No other normalization: fenced blocks, inner headings, and blank interior lines survive byte-for-byte. A parsed task with no description projects `''`; only rows without a parse (`status='invalid'`, and thin cross-repo rows in registry-backed loads) project NULL — "parsed and empty" and "unknown" stay distinguishable in SQL. `handoff` is the frontmatter key's block text; NULL when the key is absent, like every optional scalar. These columns — and `parent` after them (mw-0ssk8dg) — are appended after `error` (in that order) so readers built against the original column order are undisturbed; adding them is an additive projection change under the minting-rule idiom, not a format bump — the on-disk bytes are untouched.

Invalid files project as `tasks` rows with `status='invalid'` and `error` set — they are data, not errors. The normative queue semantics over this projection is the `ready` SQL in DESIGN §5: `open`, no unmet `needs` (unresolved counts as unmet), no live children.

## Views

The **derived projection**: one parameter table, `clock`, and thirteen named views computed from the six tables above and nothing else — `events`, `spans`, `facts`, `graph`, `asks`, `mentions`, `lineage`, `attention`, `sessions`, `flow`, `hazard`, `pulse`, and `clock` itself. They are contract (MW-S1): the reference binary registers them in every session that serves a query, a third-party reader implements them from this section, and the two are held equal by a blessed corpus. Additive under the minting-rule idiom — no on-disk byte changes, no `format` bump, no `schema` bump; a reader that ignores views loses nothing it had.

Three artifacts, three roles. **This prose is normative**: each view's rows and columns are defined here, and a reader that cannot run the reference SQL (a SQLite reader, a UI) implements the prose. **`FORMAT-views.sql`**, beside this file, is the reference implementation — the views as `CREATE VIEW` statements in DataFusion's dialect, in registration order, which the reference binary registers verbatim; where prose and SQL disagree the prose wins and the SQL has a bug. **`fixtures/conformance/expected/views/`** is the portable test: every view over the conformance store at two clock stamps, and over a two-store linked fixture as a union, rendered by the rules at the end of this section — byte-equal means conformant.

Every view is a pure function of the six tables and `clock` (MW-S2): nothing is persisted, nothing is authored, no view executes text — `verify:` is a string in `facts.has_verify` and nowhere else. Helper views (short lowercase prefixes in the SQL) are implementation and may change; only the thirteen names are contract.

### `clock`

One row, and the only parameter any view reads. The reference binary registers it as a table, never derives it from the store:

| column | meaning |
|---|---|
| `now` | timestamp — the moment every age, idle and window is measured from |
| `today` | date — `now` as a civil date, UTC |
| `source` | `override` when `now` came from `MESHWORK_TODAY`, else `system` |
| `window_days` | integer — the width of every `_w` column, from `[stats] window_days` in `config.toml`; default 7 |

`MESHWORK_TODAY` is honoured in both conforming stamp forms: a minute stamp sets `now` exactly; a date-only stamp sets `now` to that day's midnight UTC. Any other value is refused loudly — a nonconforming clock would silently shift every duration. Unset, `now` is the wall clock at UTC minute resolution, so two readers in the same minute agree.

`clock` is the **read-only idiom**: a query at a stamp asks *what these bytes say as of that moment*, against the store as it is on disk now. It is not a read at a git ref — a task closed yesterday is closed at every stamp, because the file says `done`. History is `git log` (see *Projection*); a view of the store as it was is the projection at that commit under whatever clock the reader chooses, and combining the two is the reader's own step.

### Conventions every view obeys

- **The stamp guard.** Only the two conforming stamp forms parse: `YYYY-MM-DD` and `YYYY-MM-DDTHH:MMZ`, matched as `^\d{4}-\d{2}-\d{2}(T\d{2}:\d{2}Z)?$`. Any other stamp — an offset, a seconds field, `fixed` — projects `NULL` and is invisible to every duration, window and bin (MW-S3). The guard is mandatory, not defensive: the reference engine's `to_timestamp` errors on a non-date token, and the log grammar guarantees one will occur.
- **Date-only stamps are midnight UTC** (MW-S16). `2026-07-25` as a close is `2026-07-25T00:00Z`: a task created that day and closed by a date-only stamp has lifetime 5.0 days if created five days earlier at midnight, never 4.x.
- **The closure convention** (MW-S16). `done` means **current** status. A task whose log carries `→done` but whose status is `open` was reopened and is not a closure: `facts.done_at`, `terminal_at`, `cycle_h` and `service_h` are NULL, `flow` and `pulse.done_w` do not count it, and `hazard` treats it as open. The earlier `→done` line still counts as a transition, a touch, a `reopens` and a span. One rule for every reader; the 2026-09-01 prioritization baseline's `min(→done)` differs from it by 21 tasks on one store, and that difference is the reason it is pinned here.
- **Durations are hours, as floats**; `_d` columns are days. A live task's age runs from `created` to `clock.now`; a terminal task's runs to its terminal stamp. `idle_h` is `now` minus the task's last stamped activity (`created` when it has none).
- **Scope.** Views over tasks join `repos` on `repo`, so only loaded stores count — the thin cross-repo rows a single-repo session injects to resolve dependencies never appear as tasks. Invalid rows (`status='invalid'`) are excluded everywhere except that their absence is exactly that: an invalid file contributes no event.
- **Recursion is bounded at 64** (MW-S4): the transitive walks over `needs`, `discovered-from` and `parent` stop at depth 64. `needs` and `parent` are cycle-linted; `discovered-from` is not, so a cycle there surfaces as `spawn_depth = 64`, which is a lint finding (`discovered-cycle`), never a silent cap. Lane labelling (below) stops on convergence, not on a count.
- **The window.** A `_w` column counts within `[now − window_days × 86 400 s, now]`, inclusive of the lower bound. The window is unix-second arithmetic on `clock.now` so that every view sees the same bound.
- **Union semantics.** Over several stores every view is computed once over the union, and cross-repo columns (`needs_open_foreign`, `needed_by_foreign`, `asks_in_open`, foreign mentions) are only ever non-zero there — a single-repo load reports 0 for them by construction, which is why the linked fixture exists.

### `events` — everything with a stamp

One row per stamped thing in the store: the `created` of every valid task, every `## log` entry, every comment. Columns: `gid`, `repo`, `kind`, `stamp` (as written), `at` (the stamp through the guard; NULL when nonconforming), `actor`, `from_status`, `to_status`, `note`, `ord`.

| `kind` | one row per | `actor` | `note` | `ord` |
|---|---|---|---|---|
| `created` | valid task with a `created` key | NULL | NULL | 0 |
| `transition` | log entry with a `<status>→<status>` token | the text after `claimed by ` when the note begins with it, else NULL | the note | file position |
| `close-attempt` | log free-text entry whose text begins `close attempt` | NULL | the text | file position |
| `handoff` | log free-text entry whose text begins `handoff` | the text after `handoff by `, else NULL | the text | file position |
| `note` | any other log entry | NULL | the text | file position |
| `comment` | comment | the author | the text | file position |

`actor` is populated from the only three places the store records who: comment authors, `claimed by <author>` transition notes, and `handoff by <author>` lines. Nothing else attributes — a `→done` line carries a sha, not a person.

### `spans` — time in each state

One row per transition with a parsable `at`: `gid`, `repo`, `state` (the `to_status`), `entered`, `left_at` (the next transition's `at` in file order, NULL while current), `next_state`, `hours` (`entered` to `left_at`, or to `clock.now` while open). "Blocked for 214 h" and "doing for 1.2 h" are rows here; `facts.blocked_h` and `doing_h` are their sums.

### `facts` — one row per task, every lifecycle scalar

One row per valid task in a loaded store. Identity and state: `gid`, `repo`, `id`, `title`, `status`, `category`, `seq`, `live` (open, doing or blocked), `claimed_by`. Times: `created_at` (guarded), `first_doing` and `last_doing` (min and max `→doing`), `done_at` (max `→doing→done` stamp, **only when status is `done`**), `terminal_at` (max `→done`/`→dropped`, only when status is terminal), `last_transition`, `last_activity` (max parsable stamp on any event), `last_stamp` (max stamp as written, conforming or not). Durations in hours: `age_h` (created to terminal, or to now), `queue_h` (created to first doing), `service_h` (first doing to done, only when status is done and the order holds), `cycle_h` (created to done, only when done), `blocked_h` and `doing_h` (summed spans), `idle_h`. Counts: `touches` (events other than `created`), `comments`, `actors` (distinct), `claims` (transitions with an actor), `close_attempts`, `reopens` (transitions into `open` from any status), `blocks` (transitions into `blocked`). Flags: `has_verify` (non-blank), `has_category`, `has_seq`, `waived`, `has_handoff`, `is_ask` (`to:` present).

### `graph` — structure over live `needs`

One row per valid task. `unlock` = live tasks that transitively need it; `depth` = the longest such chain; `place` = `seq`, or 999999 when unset; `inherit` = the smallest `place` among its live same-repo transitive dependents, else its own; `needs_behind` = `inherit < place` — a prerequisite placed later than work that waits on it. `lane` = the connected component of the task over live `needs` edges only (both ends live; `parent` umbrellas never join lanes), labelled by the smallest `gid` in the component; `lane_size` its member count (1 for a task in no lane). `needs_open` = unmet `needs` (target missing or non-terminal), `needs_unresolved` = targets absent from the loaded set, `needs_open_foreign` = unmet targets in another repo; `needed_by_open` = live tasks needing it, `needed_by_foreign` = those in another repo; `live_children`; `discovered` = tasks that carry `discovered-from` it; `ready` = the §5 predicate as a boolean (open, no unmet needs, no live children).

The walk over `needs` is a transitive closure bounded at 64; a diamond multiplies paths but distinct counting absorbs it. Lanes are min-label propagation over the undirected live-`needs` graph, synchronous rounds, stopping the round after no label changes — a reader may compute components any way it likes; the result is the same partition and the same labels.

### `asks` — obligations across the boundary

One row per task carrying `to:`. `from_repo`, `to_repo` (the `to:` text before `#`, or all of it), `to_ref` (as written), `title`, `status`, `created_at`, `age_h`, `idle_h`, `seq`. Answers are tasks anywhere in the loaded set carrying `answers:` naming this ask's gid, `dropped` ones excluded: `answer_gid` (the smallest such gid), `answer_status` (its current status), `answer_open` (answers that are live), `answer_done` (answers that are `done`), `answered_at` (the earliest answer's creation), `answer_done_at` (the latest done answer's close), `response_h` (ask created to `answered_at`). Two booleans carry **state, not a display rule**: `answered` = a done answer exists; `unanswered` = the ask is non-terminal and no done answer exists. An open answer is an intent, and the ask stays `unanswered` — what a digest chooses to show is the digest's rule, not this view's.

### `mentions` — every id named anywhere, resolved

One row per (source task, field, referenced task). Tokenize `handoff`, `body`, every comment and every log note on the alphabet `[a-z0-9#-]` (everything else is a separator); a token is a reference when it equals a loaded task's `gid`, or when the mentioning task's own `repo#` prefixed to it does — bare ids resolve inside their own store only. A task never mentions itself. Columns: `src_gid`, `src_repo`, `field` (`handoff` | `body` | `comment` | `log`), `ref_gid`, `times`, `first_at` (the earliest stamped mention — comments and log notes carry stamps; `handoff` and `body` do not, so NULL), `ref_status` (current), `ref_terminal_at`, `src_last_activity`, `src_handoff_at` (the source's newest `handoff` event, NULL when its block was never minted), `moved_since_activity` (the reference went terminal after the source moved: after `src_handoff_at` for a `handoff` mention that has one, else after `src_last_activity`), `edge_backed` (a declared edge of any kind exists between the two, either direction).

### `lineage` — spawn trees, umbrellas, gravity

One row per valid task. Spawn (transitive `discovered-from`, child to origin): `spawned_direct`, `spawned_total`, `spawned_live`, `spawned_done`, `spawn_depth`, `last_spawn_at` (the latest descendant's creation), `spawned_w` (descendants created in the window). Umbrella (transitive `parent`): `children_direct`, `descendants_total`, `descendants_live`, `descendants_done`, `umbrella_depth`, `last_child_at`, `children_w`. Inbound edges: `needs_in`, `relates_in`, `answers_in`, `referrers_edges` (distinct sources of any edge kind). Mentions: `mentioned_by` (distinct mentioning tasks), `mentioned_by_unlinked` (those with no edge behind the mention), `mentioned_in_talk` (in a comment or log note). `gravity` = `spawned_total + descendants_total + referrers_edges + mentioned_by`: how much of the store points at the task — a display quantity, never a rank input (MW-S12).

### `attention` and `sessions` — who touched what

**`attention`**: one row per live task — `gid`, `repo`, `id`, `status`, `idle_h`, `actors`, `touched_since_move` (distinct actors on events stamped after the task's last transition, or any actor when it has none), `first_touch_since_move`.

**`sessions`**: one row per (actor, repo) — `first_seen`, `last_seen`, `tasks_touched`, `comments`, `claims` (transitions carrying the actor).

**Sample bias, stated in the contract.** Both views see only the identities the store records: comment authors and `claimed by` notes. Half of all closed work in the field was never claimed, and most reading sessions write nothing — so these views measure a minority of actual sessions, and `touched_since_move` inherits the bias. Read them as *what was written*, never as coverage; a reader joining them to a session record must say so.

### `flow` — the daily series

One row per loaded repo per calendar day from the earliest `created_at` in the loaded set to `clock.today`: `filed`, `done`, `dropped` (tasks created, or reaching their terminal status, on that day — `done`/`dropped` by current status, per the closure convention), their running sums `filed_cum`, `done_cum`, `dropped_cum`, `backlog` (filed − done − dropped, cumulative), `backlog_delta` (that day's), `doing_eod` (doing spans open at the end of the day). Counts in named periods, never a rate, never extended past `today` (MW-S11).

### `hazard` — the discrete daily close hazard

One row per loaded repo per age bin `age_d` from 0 to 60 days: `at_risk` = tasks whose lifetime reached the bin (`age_h / 24 ≥ age_d`) **and** that have been observed through the bin's end (`now − created ≥ age_d + 1` days — equalised follow-up: a task filed this morning is at risk in no bin, one filed two days ago in bins 0 and 1); `done_in` = at-risk tasks with status `done` whose lifetime ended inside the bin; `dropped_in` the same for `dropped` — competing risks, counted apart. Pool across repos by summing numerators and denominators; the survival curve is the consumer's product of `1 − done_in / at_risk`.

### `pulse` — one row per repo, the weather

Everything above reduced to the numbers a session reads first; every count carries its denominator in the same row or in `facts`. `_w` columns are within the window.

| group | columns |
|---|---|
| state | `open_n doing_n blocked_n done_n dropped_n` (by current status) |
| flow | `filed_w` (created in window), `done_w`, `dropped_w` (terminal in window, by current status), `started_w` (distinct tasks entering `doing`), `backlog_delta_w` (filed − done − dropped), `filed_from_w` (created in window carrying `discovered-from`) |
| age | `doing_stale` (doing, idle ≥ 72 h), `open_age_med_d` (median age of open tasks, days, 1 decimal), `past_triage` (live, age ≥ 14 d), `parked_in_place` (live, has `seq`, idle ≥ 14 d) |
| structure | `ready_n`, `unlockers` (`unlock > 0`), `lanes_multi` (lanes of size > 1), `needs_behind_n`, `blocked_foreign` (live with an unmet foreign need), `blocked_unresolved` (live with an absent need), `owed_foreign` (live and needed by another repo's live task) |
| obligations | `asks_in_open`, `asks_in_oldest_d` (asks addressed *to* this repo that are `unanswered`, and the oldest's age in days), `asks_out_open`, `asks_out_oldest_d` (this repo's own `unanswered` asks) |
| friction | `close_attempts_w`, `reopens_w`, `blocks_w`, `waived_n` (done and waived), `no_verify`, `no_seq`, `no_category` (live tasks lacking each) |
| attention | `comments_w`, `actors_w` (distinct in window), `thrash_n` (live tasks with `touched_since_move ≥ 2`) |
| text and lineage | `handoff_stale_n` (tasks whose handoff mentions a terminal task), `unlinked_mentions` (mention pairs with no edge), `implicit_live_n` (unlinked mentions between two live tasks), `spawners_w` (tasks that spawned in the window), `spawned_w_max`, `umbrellas_w` (parents that gained a child in the window) |

A repo present in `repos` with no tasks projects a row of zeros and NULL medians. Over the union, `asks_in_open` for repo R counts asks in *other* stores addressed to R; in a single-repo load it is 0 by construction.

### The blessed corpus and its rendering

`fixtures/conformance/expected/views/<view>.json` holds every public view over the conformance store at `MESHWORK_TODAY=2026-08-10T12:00Z`, `expected/views/2026-09-01/` the same views at the date-only stamp `2026-09-01` (midnight UTC), and `linked/expected/views/` the union of the two linked stores `left` and `right` at the first stamp. Each file is one object: `view`, `clock` (the stamp as given), `window_days`, `columns` (in the view's column order above), `rows`.

Rendering, so that any reader can produce the same bytes: every value is a string — text as-is, integers as digits, booleans as `true`/`false`, NULL as the empty string, timestamps as `YYYY-MM-DDTHH:MM:SS` (UTC, no suffix), dates as `YYYY-MM-DD`, floating values with exactly four decimals. Rows sort by plain byte order on: `events` (`gid`, `ord`, `kind`, `stamp`); `spans` (`gid`, `entered`, `state`); `facts`, `graph`, `asks`, `lineage`, `attention` (`gid`); `mentions` (`src_gid`, `field`, `ref_gid`); `sessions` (`actor`, `repo`); `flow` (`repo`, `day`); `hazard` (`repo`, `age_d`); `pulse` (`repo`). The reference binary regenerates the corpus with `MESHWORK_BLESS=1 cargo test conformance::views_golden`; the diff is reviewed against this section before it is committed, because the corpus exists to catch the case where the two disagree.
