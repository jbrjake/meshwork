#!/usr/bin/env bash
# House numbers: warn >500 lines, fail >750 (code files; generated files exempt via .gitattributes linguist-generated).
set -u
cd "$(dirname "$0")/.."
rc=0
while IFS= read -r f; do
  n=$(wc -l < "$f")
  if   (( n > 750 )); then echo "FAIL over ceiling: $f ($n lines) — split into submodules"; rc=1
  elif (( n > 500 )); then echo "warn: $f ($n lines)"; fi
done < <(git ls-files '*.rs' '*.sh')
exit $rc
