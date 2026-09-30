#!/bin/sh
# The plugin's SessionStart hook: upgrading the plugin is the whole upgrade.
#
# A project adopts meshwork by carrying docs/meshwork/ and a .meshwork-version
# pin. When one is the session's project, this hook brings it to the release
# of the plugin the session loaded — fetches that release's binary into the
# per-machine cache if absent, rewrites the pin, rewrites the committed shim
# byte-identical to hooks/meshwork — then prints the session digest through
# that shim. The first line of output names what changed and what to commit.
# A fetch that fails changes nothing and says so in that line. A project
# whose own settings inject prime gets the change line alone, never a second
# digest. Any other project — no store, or a store without a pin (meshwork
# built from source) — is left alone in silence.
#
# Every action stays inside the project and the cache; nothing here touches
# git. Exit 0 always: a session start is never blocked over a tool upgrade.
set -u

project=${CLAUDE_PROJECT_DIR:-$PWD}
plugin=${CLAUDE_PLUGIN_ROOT:?set by Claude Code for plugin hooks}
store=$project/docs/meshwork
pin=$project/.meshwork-version
shim=$store/meshwork
canon=$plugin/hooks/meshwork

[ -d "$store" ] && [ -f "$pin" ] || exit 0
current=$(cat "$pin")

# The project's own SessionStart hook already injects prime (the pre-plugin
# ritual): report changes only, never two digests in one context.
primes_itself() {
  grep -qs 'docs/meshwork/meshwork prime' \
    "$project/.claude/settings.json" "$project/.claude/settings.local.json"
}

prime() {
  primes_itself && return 0
  "$shim" prime 2>/dev/null \
    || echo "meshwork: prime failed through docs/meshwork/meshwork (pinned $(cat "$pin"))"
}

tag=v$(sed -n 's/^ *"version": *"\([^"]*\)".*/\1/p' "$plugin/.claude-plugin/plugin.json" | head -n 1)
if [ "$tag" = v ]; then
  echo "meshwork: the plugin at $plugin states no version; this project stays at $current"
  prime
  exit 0
fi

bin=$HOME/.meshwork/versions/$tag/meshwork
changed=

# The binary first: a pin moved ahead of the binary behind it breaks every
# verb, so nothing else moves until the release is on disk.
if [ ! -x "$bin" ]; then
  case $(uname -sm) in
    "Darwin arm64")  target=aarch64-apple-darwin ;;
    "Linux aarch64") target=aarch64-unknown-linux-gnu ;;
    "Linux x86_64")  target=x86_64-unknown-linux-gnu ;;
    *)
      echo "meshwork: could not fetch release $tag — no release binary for $(uname -sm); this project stays at $current"
      prime
      exit 0
      ;;
  esac
  url="https://github.com/jbrjake/meshwork/releases/download/$tag/meshwork-$tag-$target.tar.gz"
  tmp=$(mktemp -d "${TMPDIR:-/tmp}/meshwork-fetch.XXXXXX")
  err=$tmp/fetch.err
  why=
  if ! command -v curl >/dev/null 2>&1; then
    why="curl is not installed"
  elif curl -fsSL --retry 2 -o "$tmp/release.tar.gz" "$url" </dev/null 2>"$err" \
    && tar -xzf "$tmp/release.tar.gz" -C "$tmp" 2>>"$err" \
    && [ -x "$tmp/meshwork" ]; then
    mkdir -p "$(dirname "$bin")" && mv -f "$tmp/meshwork" "$bin"
    changed="$changed binary"
  else
    why=$(tail -n 1 "$err" 2>/dev/null)
    why=${why:-$url held no meshwork binary}
  fi
  rm -rf "$tmp"
  if [ -n "$why" ]; then
    echo "meshwork: could not fetch release $tag ($why); this project stays at $current — nothing changed"
    prime
    exit 0
  fi
fi

if [ "$current" != "$tag" ]; then
  printf '%s\n' "$tag" >"$pin"
  changed="$changed .meshwork-version"
fi
if ! cmp -s "$canon" "$shim"; then
  cp "$canon" "$shim" && chmod 755 "$shim"
  changed="$changed docs/meshwork/meshwork"
fi

if [ -n "$changed" ]; then
  what=
  commit=
  case " $changed " in *" binary "*) what="fetched the binary" ;; esac
  case " $changed " in *" .meshwork-version "*) commit=".meshwork-version" ;; esac
  case " $changed " in
    *" docs/meshwork/meshwork "*) commit="${commit:+$commit and }docs/meshwork/meshwork" ;;
  esac
  if [ -n "$commit" ]; then
    what="${what:+$what, }rewrote $commit; commit $commit"
  fi
  echo "meshwork: this project is now at $tag — $what"
fi
prime
exit 0
