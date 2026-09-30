# Migrating a legacy deploy to the plugin install (read only when migrating)

Who this serves: repos wired up before the Claude Code plugin existed, or
before it carried the upgrade hook. Any one of these tells marks a legacy
deploy:

- a vendored skill copy committed at the repo's own `.claude/skills/meshwork/`
- a shim at the repo root (`./meshwork`) instead of `docs/meshwork/meshwork`
- a per-repo SessionStart hook in `.claude/settings.json` that runs `prime`
- a hook or permission rule that rebuilds the raw
  `~/.meshwork/versions/$(cat .meshwork-version)/meshwork` path

The modern endpoint (install.md defines each piece): the skill and the
SessionStart hook arrive through the plugin, the binary stays per-repo
pinned with the plugin's hook keeping the pin, the shim and the cached
binary at the plugin's release, and every invocation — sessions, hooks,
scripts — goes through the committed `docs/meshwork/meshwork` shim. Work
the steps in order: each removal has a replacement that must land first.

## 1. Install the plugin (once per machine, or once per project)

```
/plugin marketplace add jbrjake/claude-plugin-marketplace
/plugin install meshwork@jbrjake
```

Plugin installs resolve the newest release tag. The plugin ships the skill
and the upgrade hook, never the binary; the release the plugin states is
the release the project runs.

## 2. Let the hook bring the pin, the binary and the shim

A legacy repo already carries `docs/meshwork/` and `.meshwork-version`, so
the plugin's SessionStart hook acts on it at the next session start: the
plugin's release binary lands in `~/.meshwork/versions/`, the pin moves to
that release, and the canonical shim lands at `docs/meshwork/meshwork`
(rewritten if one is there, created if not). Its first line names what to
commit. A fetch that fails changes nothing and says so — fix that before
going on; nothing below works without the binary.

## 3. Retire the old shim

- Root-shim repo: `git rm meshwork`. The shim at `docs/meshwork/meshwork`
  is already the canonical one from step 2; a root copy is a second,
  stale path waiting to be called.
- Hook-only repo (no shim anywhere before step 2): nothing to remove.

Never keep a hand-edited shim. The canonical text's `MESHWORK_AUTHOR`
block is the only thing tagging agent actions with the session's author,
and it reads `CLAUDE_CODE_SESSION_ID` as well as the bridge variable —
CLI sessions export only the first, and a shim keyed on the bridge
variable alone silently stamps every agent comment and claim as the repo
owner (`default_author`). The hook rewrites any drift at every session
start, so an edit never survives anyway.

## 4. Remove the per-repo prime hook; repoint everything else at the shim

Delete the SessionStart entry that runs `prime` from `.claude/settings.json`
(and `.claude/settings.local.json`): the plugin's hook injects the digest,
and yields — change line only, no digest — while a per-repo one remains.
Any other hook or script that invokes meshwork targets the shim, never a
rebuilt versions path:

```
"$CLAUDE_PROJECT_DIR"/docs/meshwork/meshwork <verb>
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

The migration lands as a single commit: the pin and shim the hook wrote,
the root-shim removal, hook edits, permission sweep, vendored-skill removal
(the verify recasts of step 6 ride their own store-only commit). Adopter
repos often have live agent sessions sharing the checkout — check what is
already staged before each commit and stage by explicit pathspec, never a
bare `git add -A`. Then:

- Start a session: the first thing in context is the `<repo> — N open`
  digest, and nothing else names a change to commit — the plugin's hook
  found the project current and primed it through the shim.
- From an agent session, comment on the repo's migration task via the shim
  and `show` it back: the author must read `claude (<session-id>)`, not the
  human default — that is the canonical shim's `MESHWORK_AUTHOR` block at
  work.
