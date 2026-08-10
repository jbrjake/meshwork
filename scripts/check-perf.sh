#!/usr/bin/env bash
# check-perf.sh — the baseline's perf regression wall (CLAUDE-BASELINE:
# >1.5x recorded baseline fails; improvements/approved crossings reseed
# with UPDATE_BASELINE=1). Gate §7 owns the ABSOLUTE budgets (MW-C4);
# this guards drift underneath them. bench-baseline.json is data,
# reviewed like code.
set -euo pipefail
cd "$(dirname "$0")/.."

out=$(cargo test --release -- --ignored --nocapture perf:: 2>&1) || {
  echo "$out" | tail -5
  echo "check-perf: perf tests FAILED (gate §7 budget)"
  exit 1
}
medians=$(echo "$out" | grep '^perf-median ' || true)
if [ -z "$medians" ]; then
  echo "check-perf: no perf-median lines found"
  exit 1
fi

if [ "${UPDATE_BASELINE:-}" = "1" ]; then
  echo "$medians" | python3 -c '
import json, sys
data = {}
for line in sys.stdin:
    _, name, ms = line.split()
    data[name] = int(ms)
open("bench-baseline.json", "w").write(json.dumps(data, indent=2, sort_keys=True) + "\n")
'
  echo "check-perf: baseline reseeded — review the diff before committing"
  cat bench-baseline.json
  exit 0
fi

echo "$medians" | python3 -c '
import json, sys
base = json.load(open("bench-baseline.json"))
fail = 0
for line in sys.stdin:
    _, name, ms = line.split()
    ms = int(ms)
    if name not in base:
        print(f"check-perf: no baseline for {name} (UPDATE_BASELINE=1 to seed)")
        fail = 1
        continue
    wall = base[name] * 1.5
    verdict = "OK" if ms <= wall else "FAIL"
    print(f"check-perf: {verdict} {name} {ms}ms (baseline {base[name]}ms, wall {wall:.0f}ms)")
    if ms > wall:
        fail = 1
sys.exit(fail)
'
