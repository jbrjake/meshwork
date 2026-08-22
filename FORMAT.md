# FORMAT.md — the meshwork on-disk format, version 1

Normative and self-contained: a third party can implement a reader from this file without the binary. Rationale lives in DESIGN-meshwork.md; requirement IDs (`MW-*`) in REQUIREMENTS-meshwork.md. Where this file and the binary disagree, this file wins and the binary has a bug. A conformance corpus — a golden store plus its expected projection, self-checkable by any reader — lives at `fixtures/conformance/` in the meshwork repo (mw-7c6svyn).

**Versioning.** The store declares its format in `config.toml` (`format = 1`; absent means 1). The version bumps only on a *semantic* change — one that would make an old reader misread existing bytes. Additive change ships without a bump under the minting-rule idiom: new writers may mint richer forms (longer IDs, minute stamps, new log shapes), but parsers accept the old forms forever and never validate mint-time rules. A reader encountering a format newer than it knows MUST refuse loudly, never guess. Machine output carries the same number: the binary's `--json` verbs wrap results in an envelope, `{"meshwork": {"version": <binary version>, "schema": <n>}, "verb": …, "data": …}`, and the envelope's `schema` IS this format version — one contract, one number (mw-5rgq9ka). `version` identifies the producing binary and moves per release; `schema` moves only when `format` does.

## Store layout

```
docs/meshwork/               # the store root, flat
  config.toml                # presence marks a store; see below
  .gitattributes             # "/*.md merge=union\n/archive/*.md merge=union\n"
  .cache/                    # reserved scratch, .gitignore'd ("*\n!.gitignore\n");
                             #   deletable at any time, never a dependency
  <id>-<slug>.md             # one live task per file
  archive/<id>-<slug>.md     # terminal tasks (done|dropped); same format, always loaded
  attachments/<id>/<file>    # attachment payloads, plain files
```

Only `.md` files directly in the store root and in `archive/` are task files. Terminal tasks live in `archive/`; location carries no semantics beyond tidiness — every reader MUST load both directories identically. Nothing outside `docs/meshwork/` belongs to the store, and the store never references files outside its repo.

## config.toml

TOML; unknown keys are ignored (config is not the strict surface task files are).

| key | meaning |
|---|---|
| `alias` | string, required, `[a-z0-9]+` — the ID prefix for tasks minted in this store; a dash or uppercase would corrupt filename ID recovery (first two dash-segments of the stem) |
| `format` | integer — format version; absent = 1 |
| `default_author` | string — fallback identity for comments/claims |
| `[hierarchy] levels` | string list — display names for category depths; zero semantics |
| `mirror` | bool — GitHub mirror opt-in; absent = off |

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
| `docs` | list | repo-relative paths, optional `#§-anchor` |
| `attachments` | list | store-relative `attachments/<id>/<file>` paths |
| `seq` | integer | per-repo order weight; lower = sooner; gaps of 10 by convention |
| `github` | integer | mirror issue number; set once, never changes |
| `created` | string | date stamp as minted |
| `blocked-reason` | string | required non-empty when `status: blocked`; a stale reason left behind on an unblocked task is legal — at most a lint finding, never invalid |
| `claimed-by` | string | advisory claimant while doing/blocked; a claim, never a lock |
| `waived` | string | reason recorded by `close --waive` |
| `handoff` | string | authored note to the next session; meaningful only while the task is up next |

**Stamps.** Minted stamps are UTC minute resolution: `YYYY-MM-DDTHH:MMZ` (17 chars). Date-only `YYYY-MM-DD` is legal forever. These two are the only conforming forms; anything else carrying a date prefix — an offset stamp like `2026-08-06T21:47-04:00`, a seconds field, a space separator — is nonconforming (mw-8x954nr): never minted, compared as opaque text exactly as written, and a reader MAY warn. Stamps sort lexicographically, and the ordering guarantees hold only among conforming forms — in particular, date-only is a strict prefix of every minute stamp of its own day, so date-only sorts before them and max-stamp prefers the more precise entry. A nonconforming stamp still participates as written: a same-minute offset form sorts before its `Z` twin and its civil time is never computed — the cost of minting one. Last-activity of a file is the max stamp in it, always derived, never stored.

**Identity strings** (comment authors, `claimed-by`) are self-professed free strings — no accounts, no verification; an identity is a claim, recorded as claimed.

**Addressed tasks.** An *ask* is a task carrying `to:` — it lives in its author's store and is never copied anywhere. Delivery is a read-time join: a reader assembling repo R's view SHOULD surface every non-terminal ask in the loaded set whose `to:` repo-segment (the text before `#`, or the whole value) equals R, unless a non-`dropped` task anywhere in the set carries `answers:` naming the ask's gid. There is no transport, no broker, and no write into the addressee's store; an unavailable sender repo simply contributes nothing. (mw-hfvtx0s)

## Tail-section grammars

**`## log`** — append-only, one entry per line:

```
entry      = "- " transition | "- " freetext
transition = date " " status "→" status [" — " note]
freetext   = date [" " text]
date       = first whitespace-delimited token, as written
status     = "open"|"doing"|"blocked"|"done"|"dropped"
```

Parsing is positional and never validates history: token one is the date as written; a second token reading `<status>→<status>` makes the entry a transition (note = the rest, with one leading `— ` stripped if present); anything else is free text. Writers mint the em-dash separator; parsers accept its absence. Minted free-text forms include `created` and `close attempt — verify exit <N>`; minted transition notes include a block reason, `claimed by <author>`, `verify exit 0`, and `waived: <reason>` — and `→done` notes may end with an ` @ <short-sha>[+N]` anchor (repo HEAD when the close ran, `+N` uncommitted paths; mw-ntn0t32). All of it is note text under the grammar above, and extracting the anchor stays optional — but its form is normative for any consumer that does (mw-1byhnj1): a `→done` note carries an anchor iff it matches ` @ ([0-9a-f]{4,40})(?:\+([1-9][0-9]*))?$` — group 1 the short sha exactly as git minted it (lowercase hex, ≥ 4 chars), group 2 the uncommitted-path count, present only when the tree was dirty (a clean tree omits it; `+0` is never minted). A note that doesn't match simply has no anchor — absence, never an error. One agreed pattern instead of one regex per consumer; "which tree did the verify pass against" is the audit field of verify-gated closure and must not fork.

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
| `tasks` | `gid` (`repo#id`), `repo`, `id`, `title`, `status` (the five values, or `invalid`), `category`, `verify`, `waived`, `seq`, `created`, `blocked_reason`, `claimed_by`, `github`, `addressed_to` (the `to:` key), `path`, `error` (invalid rows only), `body`, `handoff` (see below) |
| `edges` | `src_gid`, `dst_gid`, `kind` (`needs`\|`parent`\|`discovered-from`\|`relates`\|`answers`), `resolved` (dst present in the loaded/registered set); `parent` edges stored child→parent; bare targets qualify with the declaring repo |
| `labels` | `gid`, `label` (exploded) |
| `comments` | `gid`, `ord` (1-based file position), `date`, `author`, `text`, `hash` (the identity hash above) |
| `log` | `gid`, `ord` (1-based file position), `date` (as written; NULL if the entry has none), `from_status`, `to_status` (NULL for free text), `note` — the `## log` grammar above, exactly |
| `repos` | `repo`, `path`, `remote`, `present` |

**`tasks.body`** and **`tasks.handoff`** carry the free text. `body` is the description section: every line between the frontmatter's closing fence and the first tail-section heading (`## log` / `## comments` at top level — a heading inside a fenced code block is content, per the tail-section grammar above), with leading and trailing whitespace trimmed. No other normalization: fenced blocks, inner headings, and blank interior lines survive byte-for-byte. A parsed task with no description projects `''`; only rows without a parse (`status='invalid'`, and thin cross-repo rows in registry-backed loads) project NULL — "parsed and empty" and "unknown" stay distinguishable in SQL. `handoff` is the frontmatter key's block text; NULL when the key is absent, like every optional scalar. Both columns are appended after `error` (in that order) so readers built against the original column order are undisturbed; adding them is an additive projection change under the minting-rule idiom, not a format bump — the on-disk bytes are untouched.

Invalid files project as `tasks` rows with `status='invalid'` and `error` set — they are data, not errors. The normative queue semantics over this projection is the `ready` SQL in DESIGN §5: `open`, no unmet `needs` (unresolved counts as unmet), no live children.
