# Installing meshwork (per project, pinned — read only when installing)

The BINARY never installs globally — no `cargo install`, no bare `meshwork`
on PATH. Each adopting project commits a `.meshwork-version` file (a release
tag) and takes its binary from that pinned release of jbrjake/meshwork. The
skill and the upgrade hook arrive through the Claude Code plugin; the plugin
never carries the binary, and nothing here needs `gh`.

## The plugin (user or project scope — the loaded plugin's release is the project's release)

```
/plugin marketplace add jbrjake/claude-plugin-marketplace
/plugin install meshwork@jbrjake
```

Install it at user scope (Claude Code's default: every project) or at
project scope (this project only), so projects can run different meshwork
versions. The plugin carries the skill, the canonical shim text and a
SessionStart hook. The hook runs in every session: in a project that carries
`docs/meshwork/` and `.meshwork-version`, it brings the project to the
plugin's release — the binary fetched into `~/.meshwork/versions/<tag>/`
when absent, the pin and the shim rewritten — then injects `prime` through
the shim. Its first line names what changed and what to commit; a fetch that
fails changes nothing and says so. Upgrading the plugin is therefore the
whole upgrade: no project is edited by hand, ever. A project without both
files is left alone.

## The binary (shared per-version cache, selected per repo — once, at adoption)

The hook acts only on a project that already has a store, and creating the
store needs the binary, so the first fetch is by hand. The pin is the
plugin's release: the `version` in `.claude-plugin/plugin.json` under the
plugin root, which sits three directories above this skill's base directory
(the path the skill printed when it loaded), `v`-prefixed. A pin behind or
ahead of the plugin is corrected at the next session start.

```bash
# once per repo (commit this): the plugin's release, from its manifest
sed -n 's/.*"version": *"\([^"]*\)".*/v\1/p' "<skill base directory>/../../../.claude-plugin/plugin.json" \
  > .meshwork-version

VER=$(cat .meshwork-version)
TARGET=aarch64-apple-darwin   # or aarch64-unknown-linux-gnu / x86_64-unknown-linux-gnu
DEST=~/.meshwork/versions/$VER
if [ ! -x "$DEST/meshwork" ]; then
  mkdir -p "$DEST"
  curl -fsSL "https://github.com/jbrjake/meshwork/releases/download/$VER/meshwork-$VER-$TARGET.tar.gz" \
    | tar -xz -C "$DEST"
fi
"$DEST/meshwork" --help >/dev/null && echo "meshwork $VER ready"
```

## The shim (`docs/meshwork/meshwork` — written by `init`, kept by the hook)

`init` writes the shim into the store it creates, byte-identical to the
canonical text the plugin ships (`hooks/meshwork` under the plugin root,
embedded in the binary), and the plugin's SessionStart hook rewrites it
whenever it drifts from that text. Never transcribe or hand-edit it. The
call that creates the store is the one call with no shim to go through, so
it takes the cache path once; every call after is `docs/meshwork/meshwork
<verb>`:

```bash
~/.meshwork/versions/"$(cat .meshwork-version)"/meshwork init
git add .meshwork-version docs/meshwork
```

`docs/meshwork/` is meshwork's only sanctioned footprint — never add files
to an adopter's repo root. The shim resolves `.meshwork-version` relative to
ITSELF (`dirname "$0"`, two levels up to the repo root), so git worktrees
and subdirectory shells both work. It supplies the agent session author,
`claude (<session id>)`, from the session's environment; an explicit `--as`
always wins, and a human shell (no session id) falls through to
`default_author` untouched. Never put `]` in an author — the comment grammar
closes on it. Hooks and scripts invoke the shim too; the raw
`~/.meshwork/versions/$(cat .meshwork-version)/meshwork` path remains the
fallback only where no repo checkout hosts a shim. Legacy deploys — a root
`./meshwork` shim, a per-repo SessionStart hook that runs prime, a hook that
rebuilds the raw versions path, or a vendored skill copy from before the
plugin — upgrade via `migrate.md`'s ordered ritual; don't improvise the
move.

## The portfolio store (the registry repo is a store too)

The repo that holds `repos.toml` — `~/Documents/code/portfolio` by default,
`MESHWORK_PORTFOLIO=<dir>` overrides — carries portfolio-scope tasks
(rulings, cross-repo seams) in its own `docs/meshwork/`, and every
`portfolio ready` / `next` / `q` reads the registry from it. It gets the
same ritual as any adopter — pin, fetch, `init` — and the plugin's hook
primes and upgrades it like any other project. Register each repo once:

```toml
[[repo]]
name = "leras"                 # the id namespace: leras#le-…
remote = "git@github.com:jbrjake/leras.git"
# the checkout defaults to ~/Documents/code/<name>; a per-machine override
# is a `name = "<path>"` line under [paths] in the gitignored repos.local.toml
```

Registry order is the `portfolio next` fallback order.

## The skill (vendored per repo — when the skill text itself must pin)

A repo that needs the skill's text locked to its binary version vendors the
release tarball into its own `.claude/skills/` — never into
`~/.claude/skills/`. The vendored copy is authoritative for that repo:

```bash
VER=$(cat .meshwork-version)
mkdir -p .claude/skills
curl -fsSL "https://github.com/jbrjake/meshwork/releases/download/$VER/meshwork-skill-$VER.tar.gz" \
  | tar -xz -C .claude/skills
git add .claude/skills/meshwork
```

Commit it with the repo. Never edit the installed copy in place — to update,
bump `.meshwork-version` and re-download both artifacts. A vendored copy
carries the skill alone: the upgrade hook is the plugin's, so a vendoring
repo keeps its own pin. Canonical source: `.claude/skills/meshwork/` in the
jbrjake/meshwork repo itself.
