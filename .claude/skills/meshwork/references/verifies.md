# Verify shapes beyond `run cargo test`

Read this when the task is an umbrella, an owner-gated hold, a deliverable
file, or when a `contains` pattern has to span lines. The grammar itself is
in SKILL.md.

## The one rule

A verify FAILS while the work is undone and passes only once it is done.
`start` red-checks it: "already green" means the verify cannot see the work,
so fix the verify before the work. `verify <id>` runs it any time and closes
nothing.

## Shapes

- **Umbrella** (a parent whose children are the work): closes when no child
  is live.
  `q "SELECT live_children FROM graph WHERE id = '<id>'" --json` reading
  `"rows":[[0]]`.
- **Owner-gated hold** (a ruling or a pick only the owner makes): a
  hand-written marker, date first so no CLI stamp can match it.
  `contains <task-file> /2026-09-01 owner approved/`. The owner writes the
  line; you never do.
- **Deliverable file** (a doc, a script, a fixture): `exists <path>`, and
  `all(exists <path>, contains <path> <marker>)` when an empty stub must not
  pass. The `exists` arm tells lint the file is the work, so
  `verify-path-missing` stays quiet until it appears.
- **Text that must go away**: `lacks <path> <literal|/regex/>`. A missing
  file refuses rather than passes.
- **Scoped test run**: `run cargo test package=<crate> target=<name>
  <filter>` spells `-p` and `--test` without a leading dash in the verify.

## Regexes

`contains <path> /<regex>/` is grep-like: `^` and `$` anchor lines and `.`
stops at a newline, so a two-phrase `.*` pattern misses a marker that
wrapped. Prefer a one-line marker; a pattern that must span a wrap leads
with `(?s)`.

## Traps

- A grep that the task's own file, or an archived task, already satisfies
  is green before the work; point it at the artifact, not the store.
- Legacy shell verifies: a piped chain reports the tail's exit, and close's
  `sh -c` has no agent-shell functions such as `rg`. Recast to the DSL.
- `run cargo test` must observe `ok. N passed` with N ≥ 1; a filter that
  matches nothing is a failure, not a pass.
