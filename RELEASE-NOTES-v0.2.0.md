# meshwork v0.2.0

Between v0.1.5 and this release, meshwork left the lab. A working repo migrated its 550-line TODO.md onto a store and ran eight sessions of real work; the portfolio layer landed so the next repos join a shared queue; and the sharp edges the pilot hit are gone. The headline numbers from the field are in the [README](https://github.com/jbrjake/meshwork#numbers): the session-start onboarding read shrank 33×, tracker busywork per session 3.7×, tracker context replay 4.6×.

## one queue across every repo

The portfolio layer is live. A portfolio is one more tiny git repo holding a `repos.toml` that names your repos; register them and:

- `meshwork portfolio ready` is the same ready queue over the union of every registered store, rows namespaced `repo#id`. `portfolio q` is the same SQL surface with a `repo` column.
- Dependencies cross repos. `needs: [beta#bz-c0r3]` in one repo resolves against the other repo's store by direct file lookup — from single-repo commands too, no portfolio load. Only a done/dropped task over there satisfies the edge; an absent or unregistered repo blocks conservatively, and `why` names the unresolved dep instead of guessing.
- `sequence.md` — a hand-ordered list of `repo#id` bullets under cosmetic tranche headings — overlays a total ordering, and `portfolio next` answers the session-start question: across everything, what single task is next? Resequencing an entire portfolio is editing one small file in one small repo, reviewed and diffed like everything else.
- Per-machine checkout paths live in a gitignored `repos.local.toml`. A registered repo missing from this machine skips with a report; a present-but-broken store is a loud error — tasks silently missing would misreport the portfolio, so nothing is ever quietly dropped.

## what the pilot broke

Eight post-migration sessions filed a stack of findings. The ones that bite during a migration are fixed in this release:

- **`import todo` was entombing work.** Indented checkboxes were folded into their parent's body as prose — exit 0, plausible count, open items buried inside a done parent that then auto-archived. Import is indent-aware now: a nested checkbox becomes a child task with its own status, at any depth, and the summary reports nesting loudly.
- **`add --batch` swallowed unknown keys.** A batch using `from:` — the spelling the help itself teaches — was accepted verbatim and produced no provenance edge. Unknown keys now refuse the whole batch atomically, nothing written, and `from:` is an accepted alias of `discovered-from:`.
- **Prose met the shell.** A multi-paragraph `--handoff` had a backticked phrase executed as command substitution. `set --handoff` and `comment`'s text argument now take `@<file>` and `-` (stdin), so prose never transits shell quoting.
- **`add --dry-run`** prints the would-be task files and writes nothing.
- **`set` grows `--cat`, `--verify`, `--title`.** Replacing a `verify:` automatically re-arms the per-clone approval gate — you approve the new text, not the memory of the old one.
- **`prime`'s weather is de-noised.** Provenance-only log stamps ("imported from TODO.md", eight times over) no longer spend the digest's byte budget saying nothing.
- **The CLI forgives the obvious.** `log`/`note` → `comment`, `done`/`finish` → `close`, `rm`/`delete`/`remove` → `drop`: each fails in a two-line error naming the working invocation, short enough to survive an agent's habitual `| tail -3`. `--category` and `--doc` work as aliases of `--cat` and `--docs`.

## capture before verifiable

Filing a task without a `verify:` is still legal — ideas are cheaper than implementations. Starting one no longer is: `start` refuses until the done-test exists, because writing it is the first unit of the work. `ready` and `prime` flag the gap with `[needs-verify]`.

## the store defends itself

- The whole concurrency story — merge without manual conflict resolution — is only true while `.gitattributes` carries the union-merge lines. A clone that loses them keeps working and fails invisibly at the first concurrent edit. `lint` now errors on the gap, `lint --fix` restores exactly the missing lines, and [FORMAT.md](https://github.com/jbrjake/meshwork/blob/main/FORMAT.md) promotes the attribute to a MUST.
- Store id aliases are constrained to `[a-z0-9]+` — a dashed alias corrupted id recovery for unparseable files. `init` refuses to write one; `lint` errors on a store that has one.

## fast, and held there

Release builds now carry perf tests in the gate: cold, `ready` over a 1K-task store and the portfolio union across 20 repos both answer in ~30ms — medians of 7 runs, against budgets of 100ms and 1s — and a committed baseline holds a 1.5× drift wall underneath those budgets.

## the skill learned the ritual

For Claude Code users, the bundled skill now: commits a two-line `./meshwork` shim at adoption, so humans, hooks, and agents all invoke the repo's pinned version without re-deriving the path; resolves the agent's session-tagged comment author from the environment automatically; and carries the task-authoring doctrine the pilot settled — imperative titles, verifies that start red, handoffs written as briefs to a stranger.

## getting it

darwin arm64, linux arm64/x86_64, windows x86_64. Pin: put `v0.2.0` in `.meshwork-version`; install to `~/.meshwork/versions/v0.2.0/` (see the meshwork adoption skill). Consuming repos upgrade by editing one file; nothing global to touch.
