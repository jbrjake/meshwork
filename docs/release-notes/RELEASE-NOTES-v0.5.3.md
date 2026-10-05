# meshwork v0.5.3

Improvements to upgrade syncing and `prime` visibility.

Upgrading the plugin now upgrades meshwork in your projects. Agents also stop asking permission for every meshwork command, and this release fixes a batch of bugs found in real use.

## easier upgrade syncing

Previously, upgrading meshwork required two steps: first, upgrade in Claude Code from `/plugin`, and then tell Claude `adopt meshwork X.Y.Z`. If you skipped that second part, it would be stuck using the old version, tantalized by new features it knew about but couldn't use.

Now, a hook runs at the start of every Claude Code session and upgrades the project to the plugin's release.

As a happy accident, this means adoption no longer adds anything to a project's `.claude/settings.json`, and installing no longer needs `gh`: the hook and the install instructions download with `curl`.

## also

- meshwork's CLI commands are all now allowed by the skill.
- `prime` puts an inbound request that is 14 days old or older at the top of its `next →` block, above your own next task, so a request from another repo can't sit unnoticed.
- `prime`'s ready count no longer includes your own outbound requests that are waiting on other projects.
- A cross-repo `needs:` pointing at an open task in another registered repo now counts as 'blocked on foreign'.
- `lint --fix` now settles random task ids if they collide post-merge. Whichever one was referenced by more tasks gets to keep the id.
- `scripts/demo-full.sh` plays a longer story across three public demo repos: tracing a bug to a library, asking it for a fix, and catching what the fix breaks.
- `prime` now counts tasks correctly when no tasks have ever been committed to git.
- `close` now runs legacy shell `verify`s with time and environment limits. A hang ends at the time limit, with the task left open and the attempt logged.
- `start` on a task that was already in progress or done ran the verify before refusing, so a `run cargo test` verify could build for minutes first. It now refuses without running anything.
- `add --batch` accepted `status: done` and wrote a closed task without running its verify. A batch with any status but `open` is now refused, and nothing is written.
- `add --batch` now checks `relates:` and `answers:`. A `relates:` naming a task in this repo that doesn't exist is now refused, and an `answers:` naming an unregistered repo draws a warning.

## getting it

The plugin tracks the newest tag: run `/plugin install meshwork@jbrjake` on a new machine, or the plugin's update command where it's already installed. The next session in each pinned project then moves it to v0.5.3. Commit the two files that session names. If a project's own `.claude/settings.json` still runs `prime` at session start, the plugin's hook prints only its change line, so you don't get two digests. Ask a session to migrate the project to drop the old hook.

The hook downloads the macOS and Linux binaries. On Windows, fetch the binary by hand.

darwin arm64, linux arm64/x86_64, windows x86_64. Pin: put `v0.5.3` in `.meshwork-version`; install to `~/.meshwork/versions/v0.5.3/` (see the meshwork adoption skill).
