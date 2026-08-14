#!/usr/bin/env bash
# Smoke gate (pre-commit, seconds): file caps, format, fast unit tests. Quiet on success (baseline rule).
set -u
cd "$(dirname "$0")/.."
if ! command -v cargo >/dev/null 2>&1; then echo "smoke: SKIP (no cargo on this machine)"; exit 0; fi

./scripts/check-file-length.sh || exit 1

# Skill budget (bytes, MW-D5 doctrine): SKILL.md loads whole into context on
# trigger; references/ load on demand and are exempt.
SKILL=.claude/skills/meshwork/SKILL.md
if [[ -f $SKILL ]]; then
  B=$(wc -c < "$SKILL")
  [[ $B -le 8192 ]] || { echo "smoke: FAIL $SKILL ${B}B > 8192B skill budget"; exit 1; }
fi
# Release-consistency guards (2026-08-14 incidents): every version the repo
# states must be derived or gate-checked — stale hardcodes shipped in
# v0.1.1..v0.3.1 (install.md pin, plugin.json manifest).
INSTALL=.claude/skills/meshwork/references/install.md
if [[ -f $INSTALL ]] && grep -qE '^echo "v[0-9]' "$INSTALL"; then
  echo "smoke: FAIL $INSTALL hardcodes a release pin — resolve via gh release view"; exit 1
fi
PLUGIN=.claude-plugin/plugin.json
if [[ -f $PLUGIN ]]; then
  PV=$(grep -m1 '"version"' "$PLUGIN" | sed 's/[^0-9.]//g')
  CV=$(grep -m1 '^version' Cargo.toml | sed 's/[^0-9.]//g')
  [[ $PV == "$CV" ]] || { echo "smoke: FAIL plugin.json version $PV != Cargo.toml $CV"; exit 1; }
fi

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
