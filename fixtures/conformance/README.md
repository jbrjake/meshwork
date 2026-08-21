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
| `tasks` | gid, repo, id, title, status, category, verify, waived, seq, created, blocked_reason, claimed_by, github, addressed_to, path, has_error, body, handoff | gid |
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
