# Migrating a legacy deploy to the plugin install (read only when migrating)

Who this serves: repos wired up before the Claude Code plugin existed. Any
one of these tells marks a legacy deploy:

- a vendored skill copy committed at the repo's own `.claude/skills/meshwork/`
- a shim at the repo root (`./meshwork`) instead of `docs/meshwork/meshwork`
- a hook or permission rule that rebuilds the raw
  `~/.meshwork/versions/$(cat .meshwork-version)/meshwork` path

The modern endpoint (install.md defines each piece): the skill arrives
user-scoped through the plugin, the binary stays per-repo pinned, and every
invocation — sessions, hooks, scripts — goes through the committed
`docs/meshwork/meshwork` shim. Work the steps in order: each removal has a
replacement that must land first.

## 1. Install the plugin (once per machine)

```
/plugin marketplace add jbrjake/claude-plugin-marketplace
/plugin install meshwork@jbrjake
```

Plugin installs resolve the newest release tag. The plugin ships the skill
only — the repo's pinned binary stays authoritative for behavior, and a
plugin newer than the repo's `.meshwork-version` defers to the repo.

## 2. Bump the pin and fetch its binary

This ritual itself ships with a release, so move the pin to a release that
carries it: re-run install.md's binary section — one command rewrites
`.meshwork-version` to the current release, the next fetches that release
into the shared `~/.meshwork/versions/` cache. Commit the pin bump with the
migration.

## 3. Land the shim at docs/meshwork/meshwork

- Root-shim repo: `git mv meshwork docs/meshwork/meshwork`, then fix the
  version lookup inside it — the pin file now sits two levels up:
  `$(dirname "$0")/../../.meshwork-version`.
- Hook-only repo (no shim anywhere): create it fresh per install.md's shim
  section.

Non-negotiable either way: the shim's `MESHWORK_AUTHOR` block. It is the
only thing tagging agent actions with the session's author — a deploy
without it silently stamps every agent comment and claim as the repo owner
(`default_author`). Diff the migrated shim against install.md's before
committing.

## 4. Repoint every hook at the shim

SessionStart — and any other hook or script that invokes meshwork — targets
the shim, never a rebuilt versions path:

```
"$CLAUDE_PROJECT_DIR"/docs/meshwork/meshwork prime 2>/dev/null || true
```

The raw `~/.meshwork/versions/$(cat …)` incantation is the pattern being
retired. It fails three observed ways: `cat` resolves against the wrong cwd
outside the repo root, a sandboxed `cat` denial kills every meshwork verb
for the whole session, and it invites version-pinned permission rules
(next step). It remains acceptable only where no repo checkout exists to
host a shim.

## 5. Sweep permission rules

An allow-rule that embeds a release tag — e.g.
`Bash(~/.meshwork/versions/v0.2.0/meshwork *)` — dies silently at the next
pin bump: every meshwork call starts prompting again. Delete any such rule
from `.claude/settings.json` and `.claude/settings.local.json`; rules
target the committed shim path instead.

## 6. Recast the store's verifies into the DSL

The pinned binary runs `verify:` DSL — `run cargo test <filter>`,
`exists <path>`, `absent <path>`, `contains <path> <lit|/regex/>`,
`all(p, …)` — and gates legacy shell text per-clone: text this clone did
not author prompts before it runs, and `lint` warns `verify-shell`. A
pre-DSL store's verifies are all shell, so sweep every open task: recast
each verify into the DSL where it fits (adopt.md step 6 is the
recast-and-red-check ritual); a check only shell can express gets
re-authored on this clone via `set --verify` — that mints this clone's
approval — and keeps the lint warning as its price.

Land the recasts as their own store-only commit, never mixed into the
migration commit below: `run cargo test` stays approval-free only while
a task file's git history touches nothing outside the store.

## 7. Delete the vendored skill copy — last

```bash
git rm -r .claude/skills/meshwork
```

Only after step 1 is confirmed — removing the vendored copy before the
plugin serves the skill leaves sessions skill-less. Exception: a repo that
deliberately pins the skill TEXT to its binary version keeps vendoring
(install.md's vendored section) and skips this step; for everyone else the
plugin copy wins and a vendored one is drift waiting to happen.

## 8. Prove it

The migration lands as a single commit: shim move or creation, hook edits,
permission sweep, vendored-skill removal, pin bump (the verify recasts of
step 6 ride their own store-only commit). Then:

- `claude -p "Without tools: quote the first line the session-start hook
  injected"` — expect the `meshwork — N open` digest, proving prime ran
  through the shim.
- From an agent session, comment on the repo's migration task via the shim
  and `show` it back: the author must read `claude (<session-id>)`, not the
  human default — that is the `MESHWORK_AUTHOR` block surviving step 3.
