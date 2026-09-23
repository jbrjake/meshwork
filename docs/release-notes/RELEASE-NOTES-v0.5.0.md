# meshwork v0.5.0

This release connects the dots in a lot of stuff that's been around from the beginning of meshwork: its name and its dependencies.

The mesh is much more tangible now, with full conversations between repos. Tasks you address to another repo now have a full lifecycle that gives both sides visibility without anyone sending any messages. When you address a task to another project, the other project creates a counterpart task on its side. Your task stays owed by that project until their counterpart finishes, answering it. Meanwhile, you can see that it's being answered and the status with a new `asks` verb that lists all of them.

The other big new feature makes the reason meshwork relies on DataFusion a little more clear. Every query session now has a derived layer of thirteen views (ages, flow, close hazard, lanes, lineage, attention) that `stats` renders as tables and `prime` reads for its weather.

Beyond those two new tentpole features, the verification language grows the two shapes agents kept asking for, and `lint` learned to read the graph.

## asks, from both sides

- `add --to <repo>` files a request
- `add --answers <gid>` files the task that answers a request
- `--relates` links softly
- `set` now also accepts `--body`, `--parent`, `--from`, and `--docs <old> <new>` to replace all properties in place
- Requests stay in the addressee's `prime` and `ready` until a task answering it is `done`. While the answer is merely open or in progress, the request reads `answered-by <gid> (<status>)`. A dropped answer resurrects the request.
- Your own requests leave your `ready` and `next`. After all, now they're someone else's problem. Instead they're listed under an `asks out` line carrying the addressee, the age, and the answer's state. `show` on a request prints it too.
- `asks` lists every inbound and outbound request, uncapped, with ages and answer states. `prime` and `ready` show the first few and name `asks` for the rest. As a convenience, typing `inbox` or `addressed` points you at it.
- `prime`'s headline counts unanswered requests and the oldest one's age.

## the derived layer, and `stats`

- `q` and `portfolio q` register thirteen views when a query names one: `events`, `spans`, `facts`, `graph`, `asks`, `mentions`, `lineage`, `attention`, `sessions`, `flow`, `hazard`, `pulse`, and the one-row `clock`
  - `MESHWORK_TODAY` sets the clock
  - `[stats] window_days` in the store config sets the width of every windowed count
  - The views are specified in FORMAT.md and pinned by a conformance corpus, so a third-party reader can implement them
- `stats [--window <n>d]` prints the pulse block, the weekly flow with week-over-week deltas, the pooled close hazard with its survival column, spans by state, the live lanes, three top-ten lists (gravity, spawners, touched since move), a mention-health line, and the placement table
  - `portfolio stats` runs the same tables over every registered store with one pulse row per repo
  - `--json` carries each table as columns and rows
- `prime` opens its weather with a pulse block: flow, queue, graph, asks, and friction
- `show` closes with two derived lines: what a task spawned and what mentions it, and every closed task its text still names
- `why` prints a task's lane, placement, and inherited place
- `q --help` lists every table with its columns and the views, and a failing query names them too

## verification dsl enhancements

- `run cargo test package=<crate> target=<name> <filter>` scopes a test run to one crate and one test target without any author text becoming a flag
- `lacks <path> <text|/regex/>` is the inverse of `contains`: a path that does not exist refuses
- `exists` accepts one `*` inside a path segment
- `contains` regexes anchor lines the way grep does: `^` and `$` match line boundaries and `.` stops at a newline
  - A pattern that must span a wrapped line leads with `(?s)`
- A `contains` or `lacks` target that is a directory is refused
- A malformed verify is refused where it is written (`add`, `set`, batch files), and `start` refuses to open work behind one
- `start`'s red-check announces itself and runs under a wall clock
  - a timeout is reported and the task still transitions
- `close` on a passing `run` verify names any uncommitted code the run covered
- `--waive` refuses a reason that still carries a template placeholder

## new linting codes for the task graph

- `needs-behind`: a prerequisite placed later than the work that needs it, naming both tasks and both places
- `handoff-cites-closed`, `implicit-edge`, `discovered-cycle`, `seq-collision`, and `close-attempts`, all derived from the views
- `verify-path-missing`: a verify that reads a file which is not in the tree
- `verify-self-satisfying` (a check aimed at the task's own file)
- `ruling-without-quote` (a claimed ruling with nothing quoted)
- The text report folds bulk rows (legacy shell verifies, implicit edges) into one line each, but now `--explain` unfolds them
  - `--json` always carries every finding
  - Findings against a task you have edited but not committed wait for the commit

## pinnable spec clauses

A spec document opts in one clause at a time: end a heading with `{#sp-<slug>}` and that section becomes a clause with a stable id. `cover <task> docs/SPEC.md#sp-<slug>` pins the clause to the task, recording the hash of its text as it reads now. The claim is "I implement this clause as it read when I said so". Retitling the heading changes nothing, but changing the words under it does.

- `lint` reports `spec-drift` on a live task whose pinned clause reads differently now, with both hashes and the command to re-pin after you have re-read it
  - A hand-written pin is an error until `cover --repin` mints a real one
- `spec list <doc>` shows every clause with its hash and who covers it
- `spec audit <doc>` answers five questions in one report: clauses nobody covers, clauses only dropped tasks cover, live tasks whose pin drifted, done tasks whose pin drifted (surfaced as re-open candidates, never reopened), and pins whose clause is gone
  - `portfolio spec audit <repo>#<doc>` counts every store's pins, which is where a cross-repo pin shows
- `prime`'s weather says how many and which live tasks the spec moved under
- Pins live in a `covers:` key and project as a `covers` table for SQL
- A clause ref can cross repos the same way a documentation link does

## archive bundles

Once a hundred closed tasks sit as loose files under `archive/`, `lint` says so and `lint --fix` folds them into a few bundle files. A bundled task still shows, queries, and takes comments like any other, and `reopen` splits it back out. The first bundle moves the store to format 2. Older binaries refuse it loudly rather than misreading it.

## smaller additions

- `set --handoff` records who wrote the handoff and when
- `prime` shows the author and the age next to the voice
- `portfolio search <term>` runs the same literal search as `search` across every registered store, grouped by repo
- `portfolio q` no longer rewrites `sequence.md`; only `ready`, `next`, and `seq` prune it
- Documentation links can cross repos: `repo#path#anchor` resolves through the registry
- `prime` ends on a two-line footer stating the four session rules
- Documentation anchors match under GitHub's slug rules as well as meshwork's, and a miss names the nearest heading
- `add` refuses a same-repo dependency, parent, or origin that does not exist, and warns about dead documentation anchors, unresolvable cross-repo targets, and shell syntax in an inline body
- The five most-hit error messages now say what to run instead

## fixes

- `q` honored `LIMIT` on every projection except one that led with `path` and carried a second column. Instead, that shape returned the whole table. It now returns the limit.
- `show` prints the path the store actually holds once a closed task has moved to `archive/`
- The session shim recognises the session id in either environment variable Claude Code sets, so comments carry the session author
- A handoff or body block containing an unindented blank line is stripped and repaired as one value, no longer cut at the blank
- `import todo` joins a wrapped headline whole and warns when a checkbox absorbs a block of indented prose as its body

## getting it

darwin arm64, linux arm64/x86_64, windows x86_64. Pin: put `v0.5.0` in `.meshwork-version`; install to `~/.meshwork/versions/v0.5.0/` (see the meshwork adoption skill).
