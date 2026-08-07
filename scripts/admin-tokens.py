#!/usr/bin/env python3
"""Tracker busywork vs work, measured from Claude Code session transcripts.

Feeds the README's "tracker busywork per session" row. Classifies every
tool call (input + its paired result) in each transcript as tracker-
administrative or work, and reports tokens (chars/4) per session:

  admin = command/file_path touches the tracker: TODO.md / HANDOFF* /
          check-todo (pre-migration world), anything meshwork — CLI
          invocations, docs/meshwork store reads+edits, ~/.meshwork
          paths (post-migration world), harness todo-list tools
          (TaskCreate/TaskUpdate/TaskList/TaskGet/TodoWrite, counted on
          both sides), and the SessionStart hook injection (prime).
  work  = every other tool call.

Matching is command/file_path ONLY — file *contents* citing TODO.md in
code comments don't count. The rule errs against meshwork: its entire
CLI surface counts as busywork.

Usage:
  python3 scripts/admin-tokens.py [--samples] <transcript.jsonl>...

Transcripts live in ~/.claude/projects/<encoded-repo-path>/*.jsonl.
Pass several pre-migration sessions to get the AVG/session line; run
post-migration sessions separately to compare eras.

First run (2026-08-07, sazed): final 10 pre-migration sessions averaged
33.3K admin tokens/session (28.8% of tool traffic, 1 busywork token per
2.5 of work); the first meshwork-tasked session spent 9.8K (8.6%, 1 per
10.6).
"""
import json
import re
import sys

PAT = re.compile(r"todo\.md|handoff|check-todo|meshwork", re.I)
ADMIN_TOOLS = {"TaskCreate", "TaskUpdate", "TaskList", "TaskGet", "TodoWrite"}


def content_chars(c):
    if isinstance(c, str):
        return len(c)
    if isinstance(c, list):
        return sum(len(b.get("text", "") or "") for b in c if isinstance(b, dict))
    return 0


def analyze(path):
    admin = work = 0
    admin_calls = work_calls = 0
    cls = {}  # tool_use_id -> is_admin
    samples = []
    for line in open(path):
        try:
            e = json.loads(line)
        except json.JSONDecodeError:
            continue
        if e.get("type") == "attachment":
            a = e.get("attachment", {})
            if a.get("type") == "hook_success" and "SessionStart" in (a.get("hookName") or ""):
                admin += len(a.get("content", "") or "")
                admin_calls += 1
            continue
        content = (e.get("message") or {}).get("content")
        if not isinstance(content, list):
            continue
        for b in content:
            if not isinstance(b, dict):
                continue
            if b.get("type") == "tool_use":
                inp = b.get("input", {})
                target = " ".join(str(inp.get(k, "")) for k in ("command", "file_path"))
                is_admin = b.get("name") in ADMIN_TOOLS or bool(PAT.search(target))
                cls[b.get("id")] = is_admin
                n = len(json.dumps(inp, ensure_ascii=False))
                if is_admin:
                    admin += n
                    admin_calls += 1
                    samples.append((b.get("name"), (inp.get("command") or inp.get("file_path") or "")[:100]))
                else:
                    work += n
                    work_calls += 1
            elif b.get("type") == "tool_result":
                n = content_chars(b.get("content"))
                if cls.get(b.get("tool_use_id"), False):
                    admin += n
                else:
                    work += n
    return admin, work, admin_calls, work_calls, samples


def main():
    show_samples = "--samples" in sys.argv
    files = [a for a in sys.argv[1:] if not a.startswith("--")]
    if not files:
        sys.exit(__doc__)
    tot_a = tot_w = 0
    print(f"{'session':<14}{'admin_tok':>10}{'work_tok':>10}{'admin%':>8}{'a_calls':>8}{'w_calls':>8}")
    for f in files:
        a, w, ac, wc, s = analyze(f)
        tot_a += a
        tot_w += w
        pct = 100 * a / (a + w) if a + w else 0
        name = f.rsplit("/", 1)[-1][:12]
        print(f"{name:<14}{a // 4:>10}{w // 4:>10}{pct:>7.1f}%{ac:>8}{wc:>8}")
        if show_samples:
            for tool, tgt in s[:40]:
                print(f"    [{tool}] {tgt}")
    if len(files) > 1:
        pct = 100 * tot_a / (tot_a + tot_w) if tot_a + tot_w else 0
        print(f"{'AVG/session':<14}{tot_a // 4 // len(files):>10}{tot_w // 4 // len(files):>10}{pct:>7.1f}%")


if __name__ == "__main__":
    main()
