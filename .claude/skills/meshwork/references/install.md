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

Hooks and commands always invoke
`~/.meshwork/versions/$(cat .meshwork-version)/meshwork`.

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
