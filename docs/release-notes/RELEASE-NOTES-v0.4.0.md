# meshwork v0.4.0

The headline is a change to the trust gate: meshwork still refuses to run verification commands that arrived from outside your clone, but it no longer asks you to approve a command you just typed yourself. Alongside that: hardening against malicious task files, full-text search, a conformance suite for third-party readers, and a documented migration for repos still on the old manual install.

## approval happens when you write the command

A task's `verify:` field holds the command that must succeed before `close` will mark the task done. Because that command runs on your machine, meshwork treats the field as untrusted input: `close` refuses to run anything nobody has approved. Until now that meant approving even commands you had just typed — write the command, then run `close --approve` to vouch for your own text. Writing the command through your own CLI (`add`, `set`, or a batch file) now counts as the approval. The explicit step remains for what it was built for: command text that arrived by merge or hand-edit, or that was edited after being approved — `lint` and `prime` flag those edits, so a change can't ride an old approval.

The README's walkthrough of this gate was re-recorded, and the release checks now replay its transcripts against a scratch store, so the demo can't drift from the shipped behavior.

## protection against malicious task files

- **Path confinement.** Every file path a task can mention — documentation references, attachments, files named in a verification check — must resolve inside the repository. A task file cannot point a read or a check at anything outside the checkout.
- **Terminal-escape stripping.** Task content is sanitized before display everywhere it is shown, including the session-start digest injected into agent context, so a crafted task file cannot send escape codes to your terminal.
- **Control characters** are rejected when writing task metadata.

## search, and more to query

- `search` finds text anywhere in the store — titles, bodies, handoff notes, comments, and logs — case-insensitively, no SQL required.
- `q` exposes more through SQL: task bodies (`tasks.body`), handoff notes (`tasks.handoff`), and parent ids now appear in the `tasks` table, and querying a table that doesn't exist reports the tables that do.

## smaller additions

- `verify <id>` runs a task's verification command and reports the result, without closing or changing anything.
- `add --body` sets the task description at creation instead of requiring a follow-up edit.
- `drop --reason` records why a task will never happen.

## lint catches more, and repairs more

New checks catch: an unclosed code fence that silently swallows the rest of the file; a closed or dropped task with no matching log entry; in-progress tasks that have gone stale or were never claimed; and prose that ended up in the wrong part of the file, which lint also moves back. `--fix` gains two repairs for merge and edit damage: colliding dependency lists and duplicated status fields. And when the store contains invalid files, `prime` now names each one instead of just counting them.

## fixes

- Markdown code fences are handled correctly everywhere: a heading quoted inside a fence is no longer mistaken for file structure or a task boundary.
- `dep add` no longer corrupts multi-line dependency lists.
- Piping output (`meshwork q ... | head`) exits cleanly instead of panicking.
- Query caching is keyed on file content rather than modification time, so a rebase or fresh checkout can never produce stale results.

## a testable spec

FORMAT.md, the specification third-party readers implement from, now includes a conformance suite: a sample store with its exact expected output, plus rulings that settle the spec's remaining ambiguities. A reader can be built and tested against the spec without ever consulting this binary.

## the migration path

The adoption skill now documents the move from the old manual install — binary and skill files copied into each repo — onto the pinned plugin install, including rewriting shell-based verification commands into the restricted predicate language introduced in v0.3.0.

## getting it

darwin arm64, linux arm64/x86_64, windows x86_64. Pin: put `v0.4.0` in `.meshwork-version`; install to `~/.meshwork/versions/v0.4.0/` (see the meshwork adoption skill).
