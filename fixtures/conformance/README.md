# FORMAT.md conformance corpus

A golden store plus its expected projection. Implement a reader from
FORMAT.md (version 1), run it over `docs/meshwork/` here, and diff your
output against `expected.json` — byte-equal means conformant. Where this
corpus and FORMAT.md disagree, the spec wins and the corpus has a bug;
file it.

## What expected.json is

One JSON object, five keys — `tasks`, `edges`, `labels`, `comments`,
`log` — each an array of rows, each row an array of strings. Columns and
order per table:

| key | columns (in order) | sort |
|---|---|---|
| `tasks` | gid, repo, id, title, status, category, verify, waived, seq, created, blocked_reason, claimed_by, github, addressed_to, path, has_error, body, handoff, parent | gid |
| `edges` | src_gid, dst_gid, kind, resolved | src_gid, kind, dst_gid |
| `labels` | gid, label | gid, label |
| `comments` | gid, ord, date, author, text, hash | gid, ord |
| `log` | gid, ord, date, from_status, to_status, note | gid, ord |

Conventions: every value renders as a string — integers as digits,
booleans as `true`/`false`, NULL/absent as the empty string. `repo` is
`conformance` (the store's directory name; no registry). `path` is
repo-relative. `has_error` replaces the `error` column: its text is
reader-defined, only its presence on invalid rows is normative. Sorts
are plain byte-order on the named columns.

## Regenerating

`MESHWORK_BLESS=1 cargo test format::conformance_corpus` rewrites
`expected.json` from the binary; the diff is then reviewed against
FORMAT.md before committing — the corpus exists to catch exactly the
case where those two disagree.

## The derived projection: `expected/views/`

FORMAT.md §Views is blessed the same way, one file per view, by
`MESHWORK_BLESS=1 cargo test conformance::views_golden`:

| path | store | clock (`MESHWORK_TODAY`) | window |
|---|---|---|---|
| `expected/views/<view>.json` | `docs/meshwork/` here | `2026-08-10T12:00Z` | 7 d |
| `expected/views/2026-09-01/<view>.json` | the same store | `2026-09-01` — the date-only form, midnight UTC | 7 d |
| `linked/expected/views/<view>.json` | the union of `linked/left` and `linked/right` (registry: `linked/portfolio/repos.toml`) | `2026-08-10T12:00Z` | 7 d |

Each file is `{view, clock, window_days, columns, rows}`; cells render by
§Views' rules (text as-is, integers as digits, booleans `true`/`false`,
NULL empty, timestamps `YYYY-MM-DDTHH:MM:SS`, dates `YYYY-MM-DD`, floats
with four decimals) and rows sort on the keys §Views names per view.
The linked pair is what makes the cross-repo columns testable — an ask
from `left` to `right` with no answer, one with an open answer, one with
a done answer; a `needs` across the boundary; a handoff citing a closed
task in the other store; a cohort of seven tasks with a known survival
curve and a censored tail; a closed-then-reopened task; a three-task lane
whose prerequisite is placed after its dependent; a nonconforming stamp.
Every one of those columns is 0 or NULL over a single store by
construction, which is why a single-store bless proves nothing about them.
