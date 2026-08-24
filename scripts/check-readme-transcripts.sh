#!/usr/bin/env bash
# Gate §9: the README's terminal transcripts replay against scratch stores
# and the pasted text must match the real binary's output (README line 79
# promises every transcript is pasted from a real run). Zero network.
# Binary: $MESHWORK_BIN > target/debug (built if missing) — the gate checks
# HEAD's binary, never a possibly-stale release build.
set -euo pipefail
cd "$(dirname "$0")/.."
if [[ -z "${MESHWORK_BIN:-}" ]]; then
  [[ -x target/debug/meshwork ]] || cargo build >/dev/null 2>&1
  MESHWORK_BIN=$PWD/target/debug/meshwork
fi
export MESHWORK_BIN
exec python3 scripts/check_readme_transcripts.py README.md
