#!/usr/bin/env bash
# verify_meshwork.sh — THE gate (DESIGN-meshwork.md §14, MW-J5). One exit 0 or it didn't pass.
# House pattern: numbered sections, each PASS / FAIL / SKIP(reason). Skips are loud, never silent.
# --strict: skips and TRACE rows still marked `planned` become failures (v1 acceptance mode, PLAN 4.3).
# No network anywhere in the gate (MW-J6): proxies cleared, mirror tests use the stub gh.
set -u
cd "$(dirname "$0")"

STRICT=0
[[ "${1:-}" == "--strict" ]] && STRICT=1

unset http_proxy https_proxy HTTP_PROXY HTTPS_PROXY ALL_PROXY all_proxy

FAILURES=0
section() { printf '\n== §%s %s\n' "$1" "$2"; }
pass()    { printf '   PASS %s\n' "${1:-}"; }
fail()    { printf '   FAIL %s\n' "${1:-}"; FAILURES=$((FAILURES + 1)); }
skip()    { # loud skip; counts as failure under --strict
  if [[ $STRICT -eq 1 ]]; then fail "SKIP not allowed under --strict: $1"; else printf '   SKIP %s\n' "$1"; fi
}

# ---------------------------------------------------------------- §1 format
section 1 "cargo fmt --check"
if cargo fmt --all -- --check >/dev/null 2>&1; then pass; else fail "formatting drift (run: cargo fmt)"; fi

# ---------------------------------------------------------------- §2 clippy
section 2 "cargo clippy --all-targets -- -D warnings"
CLIPPY_OUT=$(cargo clippy --all-targets -- -D warnings 2>&1); CLIPPY_RC=$?
if [[ $CLIPPY_RC -eq 0 ]]; then pass; else fail "clippy"; printf '%s\n' "$CLIPPY_OUT" | tail -40; fi

# ---------------------------------------------------------------- §3 tests (offline, stub gh)
section 3 "cargo test -- --include-ignored (offline; stub gh on PATH)"
STUB_PATH="$PWD/tests/bin"
TEST_PATH="$PATH"
[[ -d "$STUB_PATH" ]] && TEST_PATH="$STUB_PATH:$PATH"
TEST_OUT=$(PATH="$TEST_PATH" cargo test -- --include-ignored 2>&1); TEST_RC=$?
if [[ $TEST_RC -eq 0 ]]; then
  pass "$(printf '%s' "$TEST_OUT" | grep -Eo '[0-9]+ passed' | awk '{s+=$1} END {print s " tests"}')"
else
  fail "tests"; printf '%s\n' "$TEST_OUT" | grep -E 'FAILED|panicked|error\[|test .* \.\.\. FAIL' | head -30
fi

# ---------------------------------------------------------------- §4 coverage ≥80 (house number)
section 4 "coverage >=80% (cargo llvm-cov)"
if ! cargo llvm-cov --version >/dev/null 2>&1; then
  skip "cargo-llvm-cov not installed (cargo install cargo-llvm-cov)"
elif ! ls tests/*.rs >/dev/null 2>&1 && ! ls tests/*/main.rs >/dev/null 2>&1 && ! grep -rq '#\[test\]' src/ 2>/dev/null; then
  skip "no tests yet (bootstrap; first red test flips this on)"
else
  if PATH="$TEST_PATH" cargo llvm-cov --fail-under-lines 80 >/dev/null 2>&1; then pass; else fail "coverage <80%"; fi
fi

# ---------------------------------------------------------------- §5 file caps 500/750 (house numbers)
section 5 "file length: warn >500, fail >750 (code files)"
if ./scripts/check-file-length.sh; then pass; else fail "file(s) over 750-line ceiling"; fi

# ---------------------------------------------------------------- §6 trace completeness (MW-J5)
section 6 "TRACE.md: every MW-* MUST mapped; done-rows name real tests"
if [[ ! -f TRACE.md ]]; then
  fail "TRACE.md missing"
else
  TRACE_FAIL=0
  # every MUST id in REQUIREMENTS appears as a TRACE row
  while IFS= read -r id; do
    grep -q "^| $id " TRACE.md || { printf '   unmapped requirement: %s\n' "$id"; TRACE_FAIL=1; }
  done < <(grep -Eo 'MW-[A-Z][0-9]+ \(MUST\)' REQUIREMENTS-meshwork.md | sed 's/ (MUST)//' | sort -u)
  # done-rows must cite tests that exist (gate-satisfied rows say "gate")
  TEST_LIST=$(PATH="$TEST_PATH" cargo test -- --list 2>/dev/null || true)
  while IFS= read -r row; do
    tests=$(printf '%s' "$row" | grep -Eo '`[a-z_]+::[a-z_0-9]+`' | tr -d '`')
    if [[ -z "$tests" ]]; then
      printf '%s' "$row" | grep -qi 'gate\|pilot\|checklist' || { printf '   done-row cites no test: %s\n' "$(printf '%s' "$row" | cut -c1-60)"; TRACE_FAIL=1; }
      continue
    fi
    for t in $tests; do
      printf '%s' "$TEST_LIST" | grep -q "$t" || { printf '   phantom test in done-row: %s\n' "$t"; TRACE_FAIL=1; }
    done
  done < <(grep '^| MW-' TRACE.md | grep '| done |')
  # planned rows: fine normally, fatal under --strict
  PLANNED=$(grep -c '| planned |' TRACE.md || true)
  if [[ $STRICT -eq 1 && ${PLANNED:-0} -gt 0 ]]; then printf '   %d rows still planned\n' "$PLANNED"; TRACE_FAIL=1; fi
  if [[ $TRACE_FAIL -eq 0 ]]; then pass "(${PLANNED:-0} rows planned)"; else fail "trace"; fi
fi

# ---------------------------------------------------------------- §7 perf (MW-C4; N>=7 median; arrives M2)
section 7 "perf: ready <100ms @1K tasks, portfolio <1s @20 repos"
if PATH="$TEST_PATH" cargo test -- --list 2>/dev/null | grep -q '^perf::'; then
  if PATH="$TEST_PATH" cargo test --release -- --ignored perf:: 2>&1 | tail -1 | grep -q 'test result: ok'; then pass; else fail "perf regression"; fi
else
  skip "no perf:: tests yet (pending M2, PLAN 2.5)"
fi

# ---------------------------------------------------------------- §8 self-host (arrives M1)
section 8 "self-host: meshwork lint + prime on this repo's own store"
if [[ -d meshwork/tasks ]]; then
  BIN=target/debug/meshwork
  [[ -x $BIN ]] || cargo build >/dev/null 2>&1
  if "$BIN" lint >/dev/null 2>&1; then pass "lint"; else fail "self lint"; fi
  PRIME_BYTES=$("$BIN" prime 2>/dev/null | wc -c)
  if (( PRIME_BYTES > 0 && PRIME_BYTES <= 6144 )); then pass "prime ${PRIME_BYTES}B <= 6144B"; else fail "prime ${PRIME_BYTES}B (budget 6144B, MW-D3)"; fi
else
  skip "no self store yet (pending M1 dogfood, PLAN 1.8)"
fi

# ----------------------------------------------------------------
printf '\n'
if [[ $FAILURES -eq 0 ]]; then
  echo "verify_meshwork: ALL SECTIONS PASS$( [[ $STRICT -eq 1 ]] && echo ' (strict)' )"
  exit 0
else
  echo "verify_meshwork: $FAILURES section(s) FAILED"
  exit 1
fi
