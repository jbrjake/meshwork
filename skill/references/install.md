# Installing meshwork (per project, pinned — read only when installing)

Owner ruling: NO global `cargo install`, no bare `meshwork` on PATH — each
consuming repo chooses its own version. Releases are built by tag-push CI on
jbrjake/meshwork (darwin arm64 today).

```bash
# once per repo: pin the version (commit this file)
echo "v0.1.0" > .meshwork-version

# install the pinned version into the shared cache if missing
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

## Installing / updating this skill

Canonical home: `skill/` in the jbrjake/meshwork repo (ships with each release).
Update the installed copy from the repo; never edit it in place:

```bash
mkdir -p ~/.claude/skills/meshwork/references
for f in SKILL.md references/install.md references/adopt.md; do
  gh api -H "Accept: application/vnd.github.raw" \
    "repos/jbrjake/meshwork/contents/skill/$f" > ~/.claude/skills/meshwork/$f
done
```
