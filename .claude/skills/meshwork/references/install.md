# Installing meshwork (per project, pinned — read only when installing)

Owner ruling: NOTHING meshwork-related installs globally — no `cargo install`,
no bare `meshwork` on PATH, no user-level skill. Each consuming repo commits a
`.meshwork-version` file (a release tag) and both artifacts — binary and skill
— come from that same pinned release of jbrjake/meshwork.

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

Nine pilot sessions re-derived the pinned path in every command
(`M=~/.meshwork/versions/$(cat .meshwork-version)/meshwork && $M …`) —
boilerplate in every transcript line (mw-we7g0k3). Commit a two-line shim
instead, and every invocation becomes `./meshwork <verb>`:

```bash
printf '#!/bin/sh\nexec ~/.meshwork/versions/"$(cat "$(dirname "$0")/.meshwork-version")"/meshwork "$@"\n' > meshwork
chmod +x meshwork
git add meshwork
```

The shim resolves `.meshwork-version` relative to ITSELF (`dirname "$0"`),
so git worktrees and subdirectory shells both work. Hooks and scripts
invoke the shim too; the raw
`~/.meshwork/versions/$(cat .meshwork-version)/meshwork` path remains the
fallback where a repo checkout isn't available.

## The skill (into THIS repo, committed)

The skill is versioned with the binary and installs into the consuming repo's
own `.claude/skills/` — never into `~/.claude/skills/`:

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
