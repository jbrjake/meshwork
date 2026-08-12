# Installing meshwork (per project, pinned — read only when installing)

Owner ruling: the BINARY never installs globally — no `cargo install`, no bare
`meshwork` on PATH. Each consuming repo commits a `.meshwork-version` file (a
release tag) and takes its binary from that pinned release of jbrjake/meshwork.
The skill has exactly one sanctioned user-scoped path — the Claude Code plugin
below — and the plugin distributes the skill surface only, never the binary.

## The binary (shared per-version cache, selected per repo)

```bash
# once per repo: pin the version (commit this file)
echo "v0.1.1" > .meshwork-version

VER=$(cat .meshwork-version)
DEST=~/.meshwork/versions/$VER
if [ ! -x "$DEST/meshwork" ]; then
  mkdir -p "$DEST"
  gh release download "$VER" -R jbrjake/meshwork \
    -p "*aarch64-apple-darwin.tar.gz" -O - | tar -xz -C "$DEST"
fi
"$DEST/meshwork" --help >/dev/null && echo "meshwork $VER ready"
```

## The shim (committed; what sessions actually run)

Commit a small shim so every invocation is just `./meshwork <verb>` —
never re-derive the pinned path inline:

```bash
printf '%s\n' \
  '#!/bin/sh' \
  '# agent sessions get a session-tagged author; explicit --as still wins' \
  'if [ -z "$MESHWORK_AUTHOR" ] && [ -n "$CLAUDE_CODE_BRIDGE_SESSION_ID" ]; then' \
  '  export MESHWORK_AUTHOR="claude ($CLAUDE_CODE_BRIDGE_SESSION_ID)"' \
  'fi' \
  'exec ~/.meshwork/versions/"$(cat "$(dirname "$0")/.meshwork-version")"/meshwork "$@"' \
  > meshwork
chmod +x meshwork
git add meshwork
```

The shim resolves `.meshwork-version` relative to ITSELF (`dirname "$0"`),
so git worktrees and subdirectory shells both work. Hooks and scripts
invoke the shim too; the raw
`~/.meshwork/versions/$(cat .meshwork-version)/meshwork` path remains the
fallback where a repo checkout isn't available.

An explicit `--as` always wins, and a human shell (no session id) falls
through to `default_author` untouched. Never put `]` in an author — the
comment grammar closes on it.

## The skill (plugin, user-scoped — the default)

```
/plugin marketplace add jbrjake/claude-plugin-marketplace
/plugin install meshwork@jbrjake
```

The plugin carries only the skill. The binary stays per-repo pinned as above,
and every command the skill issues goes through the repo's committed
`./meshwork` shim — so the release a repo pins is the release that runs. A
plugin newer than a repo's `.meshwork-version` defers to that repo's version;
never assume the plugin's own feature set.

## The skill (vendored per repo — when the skill text itself must pin)

A repo that needs the skill's text locked to its binary version vendors the
release tarball into its own `.claude/skills/` — never into
`~/.claude/skills/`. The vendored copy is authoritative for that repo:

```bash
VER=$(cat .meshwork-version)
mkdir -p .claude/skills
gh release download "$VER" -R jbrjake/meshwork \
  -p "meshwork-skill-*.tar.gz" -O - | tar -xz -C .claude/skills
git add .claude/skills/meshwork
```

Commit it with the repo. Never edit the installed copy in place — to update,
bump `.meshwork-version` and re-download both artifacts. Canonical source:
`.claude/skills/meshwork/` in the jbrjake/meshwork repo itself.
