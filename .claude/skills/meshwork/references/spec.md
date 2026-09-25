# Pinning tasks to spec clauses

A task that implements a paragraph of a specification pins that paragraph, so
meshwork can say when the words under it change. Nothing here needs version
control to detect a move; git only adds who moved it.

## Anchor the clause

A spec document opts in one clause at a time. End a heading with
`{#sp-<slug>}` — lowercase letters, digits, hyphens:

```
## Spill batches {#sp-spill-batch}
Batches flush at 64 rows or 250 ms, whichever comes first.
```

The clause is the section under that heading up to the next heading of the
same or a shallower level. Its hash is taken from the text below the heading
line, so retitling the heading changes nothing and any edit to the words does.
Never renumber or reuse an anchor id.

## Pin, list, audit

```
meshwork cover <task> docs/SPEC.md#sp-spill-batch      # pins ref + hash as it reads now
meshwork cover <task> <repo>#docs/SPEC.md#sp-<slug>    # a clause in a registered sibling
meshwork spec list docs/SPEC.md                        # every clause, its hash, who covers it
meshwork spec audit docs/SPEC.md                       # one report, five questions
meshwork portfolio spec audit <repo>#docs/SPEC.md      # every store's pins counted
```

`spec audit` answers: clauses nobody covers; clauses only dropped tasks cover;
live tasks whose pin drifted; done tasks whose pin drifted (re-open
candidates — it never reopens them); pins whose clause is gone.

## When the spec moves

- `lint` reports `spec-drift` on a live task whose clause reads differently
  now, with both hashes and the repin command. `prime`'s weather says how
  many live tasks the spec moved under.
- Re-read the clause, adjust the task if the work changed, then
  `meshwork cover <task> --repin` (every pin of the task) or
  `meshwork cover <task> <ref> --repin` (one).
- A pin is written only by `cover`. A hand-written `covers:` entry is a lint
  error on a live task until `--repin` mints a real one.

## In SQL

Pins are `covers:` rows in the task file and the `covers` table in `q`:
`q "SELECT gid, ref, sha, resolved FROM covers"`. `resolved` is false when
the ref's document or anchor is missing.
