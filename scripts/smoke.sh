#!/usr/bin/env bash
# Smoke gate (pre-commit, seconds): file caps, format, fast unit tests. Quiet on success (baseline rule).
set -u
cd "$(dirname "$0")/.."
if ! command -v cargo >/dev/null 2>&1; then echo "smoke: SKIP (no cargo on this machine)"; exit 0; fi

./scripts/check-file-length.sh || exit 1
cargo fmt --all -- --check >/dev/null 2>&1 || { echo "smoke: FAIL formatting (run: cargo fmt)"; exit 1; }

# Fast unit tier only (unit tests live in src/ modules of the binary). No RUSTFLAGS here:
# flipping flags invalidates the whole dep cache per commit; -D warnings is gate §2's job.
OUT=$(cargo test --bins 2>&1); RC=$?
if [[ $RC -ne 0 ]]; then
  echo "smoke: FAIL tests"
  printf '%s\n' "$OUT" | grep -E 'FAILED|panicked|error\[|test .* \.\.\. FAIL' | head -20
  exit 1
fi
N=$(printf '%s' "$OUT" | grep -Eo '[0-9]+ passed' | awk '{s+=$1} END {print s+0}')
echo "smoke: OK ($N tests)"
