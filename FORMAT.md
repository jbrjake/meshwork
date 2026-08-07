# FORMAT.md — the meshwork on-disk format, version 1

Normative and self-contained: a third party can implement a reader from this file without the binary. Rationale lives in DESIGN-meshwork.md; requirement IDs (`MW-*`) in REQUIREMENTS-meshwork.md. Where this file and the binary disagree, this file wins and the binary has a bug.

**Versioning.** The store declares its format in `config.toml` (`format = 1`; absent means 1). The version bumps only on a *semantic* change — one that would make an old reader misread existing bytes. Additive change ships without a bump under the minting-rule idiom: new writers may mint richer forms (longer IDs, minute stamps, new log shapes), but parsers accept the old forms forever and never validate mint-time rules. A reader encountering a format newer than it knows MUST refuse loudly, never guess.

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
| `alias` | string, required — the ID prefix for tasks minted in this store |
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
| `verify` | string | close-gate shell command — untrusted input (MW-E5) |
| `docs` | list | repo-relative paths, optional `#§-anchor` |
| `attachments` | list | store-relative `attachments/<id>/<file>` paths |
| `seq` | integer | per-repo order weight; lower = sooner; gaps of 10 by convention |
| `github` | integer | mirror issue number; set once, never changes |
| `created` | string | date stamp as minted |
| `blocked-reason` | string | required non-empty iff `status: blocked` |
| `claimed-by` | string | advisory claimant while doing/blocked; a claim, never a lock |
| `waived` | string | reason recorded by `close --waive` |
| `handoff` | string | authored note to the next session; meaningful only while the task is up next |

**Stamps.** Minted stamps are UTC minute resolution: `YYYY-MM-DDTHH:MMZ` (17 chars). Date-only `YYYY-MM-DD` is legal forever. Stamps sort lexicographically; last-activity of a file is the max stamp in it, always derived, never stored.

**Identity strings** (comment authors, `claimed-by`) are self-professed free strings — no accounts, no verification; an identity is a claim, recorded as claimed.

## Tail-section grammars

**`## log`** — append-only, one entry per line:

```
entry      = "- " transition | "- " freetext
transition = date " " status "→" status [" — " note]
freetext   = date [" " text]
date       = first whitespace-delimited token, as written
status     = "open"|"doing"|"blocked"|"done"|"dropped"
```

Parsing is positional and never validates history: token one is the date as written; a second token reading `<status>→<status>` makes the entry a transition (note = the rest, with one leading `— ` stripped if present); anything else is free text. Writers mint the em-dash separator; parsers accept its absence. Minted free-text forms include `created` and `close attempt — verify exit <N>`; minted transition notes include a block reason, `claimed by <author>`, `verify exit 0`, and `waived: <reason>`.

**`## comments`** — append-only, one entry per line:

```
entry = "- " date " [" author "] " text
```

Neither date nor author may be empty; a nonconforming entry is a warning, and the line is skipped, not fatal. A comment's identity is the tuple (date, author, text); a canonical content hash over that tuple (mirror dedup, replication) is specified when it lands (mw-xvtf5jx) — until then the tuple itself is the identity.

## Merge semantics

The store is safe under concurrent edits in separate clones: creation is file-per-task; status edits touch one frontmatter line; log/comments append at end-of-file under the committed `merge=union` attribute. Union's known failure mode — both sides editing the same frontmatter line — produces duplicate YAML keys, which strict parsing rejects into a loud invalid row; repair is mechanical (`lint --fix`). Duplicate IDs minted by parallel clones are detected post-merge and re-slugged. No locks, no daemon, no merge driver beyond the git built-in.

## Projection

The file→row projection is **stable and deterministic**: the store at commit X projects to exactly one set of rows, independent of platform, locale, load order, or wall clock. Each commit touching `docs/meshwork/` is therefore a well-defined delta of the task graph, and **git history is the change stream** — there is no other journal, and no verb will ever wrap one. Per-task history is the filename-prefix idiom:

```
git log -- "docs/meshwork/<id>-*" "docs/meshwork/archive/<id>-*"
```

Any external reader, UI layer, or incremental engine builds on (projection at X) + (commits after X). `.cache/tasks.jsonl` is reserved as an optional materialization of this projection; it is never authoritative and deleting it is always safe.

The projection is six tables. `repo` is the registry name from the portfolio's `repos.toml`, defaulting to the repo directory's name; `gid` is `repo#id` and is unique across a loaded set:

| table | columns |
|---|---|
| `tasks` | `gid` (`repo#id`), `repo`, `id`, `title`, `status` (the five values, or `invalid`), `category`, `verify`, `waived`, `seq`, `created`, `blocked_reason`, `claimed_by`, `github`, `path`, `error` (invalid rows only) |
| `edges` | `src_gid`, `dst_gid`, `kind` (`needs`\|`parent`\|`discovered-from`\|`relates`), `resolved` (dst present in the loaded/registered set); `parent` edges stored child→parent; bare targets qualify with the declaring repo |
| `labels` | `gid`, `label` (exploded) |
| `comments` | `gid`, `ord` (1-based file position), `date`, `author`, `text` |
| `log` | `gid`, `ord` (1-based file position), `date` (as written; NULL if the entry has none), `from_status`, `to_status` (NULL for free text), `note` — the `## log` grammar above, exactly |
| `repos` | `repo`, `path`, `remote`, `present` |

Invalid files project as `tasks` rows with `status='invalid'` and `error` set — they are data, not errors. The normative queue semantics over this projection is the `ready` SQL in DESIGN §5: `open`, no unmet `needs` (unresolved counts as unmet), no live children.
