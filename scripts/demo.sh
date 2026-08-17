#!/usr/bin/env bash
# The 60-second demo (mw-78nabpd): the whole loop on a scratch repo, one
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

run() { printf '\n$ meshwork %s\n' "$*"; "$BIN" "$@"; }

run init
run add "Reproduce the spill cliff" --cat engine/spill --verify "test -f repro.log"
ID=$("$BIN" q "SELECT id FROM tasks" --json | sed -n 's/.*"rows":\[\["\([^"]*\)".*/\1/p')
run add "Fix spill batch sizing" --cat engine/spill --needs "$ID" --seq 10 --verify "true"
run ready
run start "$ID" --as demo
run comment "$ID" --as demo "cliff reproduces at batch=64k"
touch repro.log
run close "$ID" --approve
run prime
run q "SELECT id, status, category FROM tasks ORDER BY id"

printf '\ndemo: done — the scratch repo is deleted on exit; your repo was never touched.\n'
