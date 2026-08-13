#!/usr/bin/env python3
"""Batch exact-string substitutions from a JSON spec — the sanctioned alternative
to editing files through python/perl/sed heredocs (which can never be pre-approved).

Usage:  python3 scripts/batch_edit.py <spec.json>     # run from the repo root

Spec:   [{"file": "relative/path.rs",
          "subs": [["old", "new", expected_count], ...]}, ...]

- `expected_count` may be omitted and then defaults to 1 (Edit-tool semantics).
- Substitutions apply in order; each count is checked against the text as already
  transformed by the previous subs.
- Everything is validated before ANYTHING is written: on the first mismatch or
  bad path, no file on disk changes and the exit code is 1.
- Paths must stay inside the working directory; `.git/` is refused.
"""
import json
import sys
from pathlib import Path


def fail(msg):
    print(f"batch_edit: {msg}", file=sys.stderr)
    sys.exit(1)


def main():
    if len(sys.argv) != 2:
        fail("usage: python3 scripts/batch_edit.py <spec.json>")
    try:
        spec = json.loads(Path(sys.argv[1]).read_text())
    except (OSError, json.JSONDecodeError) as e:
        fail(f"cannot read spec: {e}")
    if not isinstance(spec, list) or not spec:
        fail("spec must be a non-empty JSON array of {file, subs} objects")

    root = Path.cwd().resolve()
    texts = {}
    applied = {}

    for i, entry in enumerate(spec):
        rel = entry.get("file") if isinstance(entry, dict) else None
        subs = entry.get("subs") if isinstance(entry, dict) else None
        if not isinstance(rel, str) or not isinstance(subs, list) or not subs:
            fail(f"entry {i}: need {{'file': str, 'subs': [[old, new, count?], ...]}}")
        p = (root / rel).resolve()
        if root not in p.parents:
            fail(f"entry {i}: {rel!r} escapes the working directory")
        if ".git" in p.relative_to(root).parts:
            fail(f"entry {i}: refusing to touch {rel!r} (under .git/)")
        if p not in texts:
            try:
                texts[p] = p.read_text()
            except OSError as e:
                fail(f"entry {i}: cannot read {rel!r}: {e}")
            applied[p] = 0
        s = texts[p]
        for j, sub in enumerate(subs):
            if not isinstance(sub, list) or len(sub) not in (2, 3):
                fail(f"{rel} sub {j}: need [old, new] or [old, new, expected_count]")
            old, new = sub[0], sub[1]
            want = sub[2] if len(sub) == 3 else 1
            if (not isinstance(old, str) or not old or not isinstance(new, str)
                    or not isinstance(want, int) or want < 1):
                fail(f"{rel} sub {j}: old/new must be strings (old non-empty), count a positive int")
            n = s.count(old)
            if n != want:
                fail(f"{rel} sub {j}: expected {want} occurrence(s) of {old[:60]!r}, "
                     f"found {n}; nothing written")
            s = s.replace(old, new)
            applied[p] += 1
        texts[p] = s

    for p, s in texts.items():
        p.write_text(s)
        print(f"ok {p.relative_to(root)} ({applied[p]} subs)")


if __name__ == "__main__":
    main()
