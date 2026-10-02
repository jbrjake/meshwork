#!/usr/bin/env bash
# The 60-second demo: the README's quick-start loop on a scratch repo, one
# command, zero network. Run from a clone: ./scripts/demo.sh
# Binary resolution: $MESHWORK_BIN > target/release > target/debug >
# the repo-pinned version > meshwork on PATH.
set -euo pipefail
cd "$(dirname "$0")/.."

if [[ -n "${MESHWORK_BIN:-}" ]]; then BIN=$MESHWORK_BIN
elif [[ -x target/release/meshwork ]]; then BIN=$PWD/target/release/meshwork
elif [[ -x target/debug/meshwork ]]; then BIN=$PWD/target/debug/meshwork
elif [[ -f .meshwork-version && -x ~/.meshwork/versions/$(cat .meshwork-version)/meshwork ]]; then
  BIN=~/.meshwork/versions/$(cat .meshwork-version)/meshwork
elif command -v meshwork >/dev/null; then BIN=$(command -v meshwork)
else
  echo "demo: no meshwork binary — cargo build, or set MESHWORK_BIN" >&2
  exit 1
fi

DEMO=$(mktemp -d)
trap 'rm -rf "$DEMO"' EXIT
cd "$DEMO"
git init -q demo && cd demo
git config user.name "Demo" && git config user.email demo@example.invalid
# The project under the demo: one config file, which the second task's fix edits.
printf 'batch_rows = 65536\n' > spill.toml
git add spill.toml && git commit -qm "feat(engine): spill config"

# Echo a command the way it would be typed: an argument with a space or a
# shell metacharacter in it is double-quoted.
show() {
  local a line=
  for a; do
    if [[ $a =~ [^A-Za-z0-9_./:=@%+,-] ]]; then line+=" \"$a\""; else line+=" $a"; fi
  done
  printf '\n$%s\n' "$line"
}
run() { show meshwork "$@"; "$BIN" "$@"; }
# `add` prints the new id on its first line; MINTED keeps it for later steps.
mint() {
  local out
  show meshwork "$@"
  out=$("$BIN" "$@")
  printf '%s\n' "$out"
  MINTED=${out%%$'\n'*}
}
sh_() { show "$@"; "$@"; }

run init
mint add "Reproduce the spill cliff" --cat engine/spill --verify "exists repro.log"
ID=$MINTED
mint add "Fix spill batch sizing" --cat engine/spill --needs "$ID" \
  --verify "contains spill.toml /batch_rows = 16384/"
run prime
run start "$ID" --as demo
run comment "$ID" --as demo "cliff reproduces at batch=64k"
sh_ touch repro.log
run close "$ID"
run ready
run q "SELECT category, status, count(*) AS n FROM tasks GROUP BY category, status ORDER BY status"
sh_ cat docs/meshwork/archive/"$ID"-*.md

printf '\ndemo: done — the scratch repo is deleted on exit; your repo was never touched.\n'
