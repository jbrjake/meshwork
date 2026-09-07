#!/usr/bin/env python3
"""Flatten a Claude Code session transcript (.jsonl) into readable text. Read-only.

    python3 scripts/render_session.py <session.jsonl | session-id-prefix> [--mw] [--max-result N]
        [--max-text N] [--from-turn N] [--to-turn N] [--project SLUG]

Default: every human prompt, assistant text, tool call (command / file path), and tool result,
each truncated. --mw keeps only meshwork-relevant events plus every human prompt and every
assistant text, so a 3 MB session reads in a few hundred lines. A bare id prefix is resolved
under ~/.claude/projects (CLAUDE_PROJECTS) — pass --project to disambiguate.

Turn numbers count human prompts, so `--from-turn 5 --to-turn 7` isolates one exchange.
"""
import argparse
import glob
import json
import os
import re
import sys

PROJECTS_DIR = os.path.expanduser(os.environ.get("CLAUDE_PROJECTS", "~/.claude/projects"))
SYSREM = re.compile(r"<system-reminder>.*?</system-reminder>", re.S)
MW_RX = re.compile(r"meshwork|\b[a-z]{2}-(?=[a-z0-9]*\d)[a-z0-9]{4,7}\b|docs/meshwork/", re.I)
WRAPPERS = ("<command-name>", "<local-command-caveat>", "<local-command-stdout>", "<bash-input>",
            "<bash-stdout>", "<bash-stderr>")


def resolve(arg, project):
    if os.path.exists(arg):
        return arg
    pat = os.path.join(PROJECTS_DIR, project or "*", arg + "*.jsonl")
    hits = glob.glob(pat)
    if len(hits) != 1:
        sys.exit(f"{len(hits)} transcripts match {pat}; pass a path or --project")
    return hits[0]


def clip(s, n):
    s = s if isinstance(s, str) else json.dumps(s)
    return s if len(s) <= n else s[:n] + f" …[+{len(s) - n} chars]"


def text_of(content):
    if isinstance(content, str):
        return content
    if isinstance(content, list):
        return "\n".join(b.get("text", "") for b in content if isinstance(b, dict) and b.get("type") == "text")
    return ""


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("session")
    ap.add_argument("--project")
    ap.add_argument("--mw", action="store_true", help="only meshwork-relevant tool events")
    ap.add_argument("--max-result", type=int, default=700)
    ap.add_argument("--max-text", type=int, default=1200)
    ap.add_argument("--max-cmd", type=int, default=400)
    ap.add_argument("--from-turn", type=int, default=0)
    ap.add_argument("--to-turn", type=int, default=10 ** 9)
    ap.add_argument("--no-thinking", action="store_true", default=True)
    args = ap.parse_args()
    path = resolve(args.session, args.project)
    print(f"# {path}")
    tools = {}     # tool_use_id -> (name, is_mw, label)
    queued = set()  # prompts typed mid-turn (queue-operation records), printed once
    turn = 0
    title = ""
    for line in open(path, encoding="utf-8", errors="replace"):
        try:
            rec = json.loads(line)
        except Exception:
            continue
        typ = rec.get("type")
        if typ == "ai-title":
            title = rec.get("aiTitle") or title
            continue
        if typ == "queue-operation" and rec.get("operation") == "enqueue":
            t = (rec.get("content") or "").strip()
            if t and t not in queued:
                queued.add(t)
                turn += 1
                if args.from_turn <= turn <= args.to_turn:
                    print(f"\n===== USER turn {turn} @{(rec.get('timestamp') or '')[11:16]} (typed mid-turn) =====\n{clip(t, 4000)}\n")
            continue
        if typ not in ("user", "assistant"):
            continue
        side = "  [sub]" if rec.get("isSidechain") else ""
        ts = (rec.get("timestamp") or "")[11:16]
        msg = rec.get("message") or {}
        content = msg.get("content")
        if typ == "user":
            if isinstance(content, list) and any(isinstance(b, dict) and b.get("type") == "tool_result" for b in content):
                if not (args.from_turn <= turn <= args.to_turn):
                    continue
                for b in content:
                    if not isinstance(b, dict) or b.get("type") != "tool_result":
                        continue
                    name, is_mw, label = tools.get(b.get("tool_use_id"), ("?", False, ""))
                    if args.mw and not is_mw:
                        continue
                    body = text_of(b.get("content"))
                    flag = " ERROR" if b.get("is_error") else ""
                    print(f"    ← {name}{flag}{side}: {clip(body.strip(), args.max_result)}")
                continue
            if rec.get("isMeta") or rec.get("isSidechain"):
                continue
            t = SYSREM.sub("", text_of(content)).strip()
            if not t or t.startswith(WRAPPERS) or t in queued:
                continue
            turn += 1
            if not (args.from_turn <= turn <= args.to_turn):
                continue
            print(f"\n===== USER turn {turn} @{ts} =====\n{clip(t, 4000)}\n")
            continue
        # assistant
        if not (args.from_turn <= turn <= args.to_turn):
            # still need tool ids for later results
            for b in content or []:
                if isinstance(b, dict) and b.get("type") == "tool_use":
                    tools[b.get("id")] = (b.get("name"), False, "")
            continue
        for b in content or []:
            if not isinstance(b, dict):
                continue
            bt = b.get("type")
            if bt == "text":
                t = b.get("text", "").strip()
                if t:
                    print(f"  [assistant{side} @{ts}] {clip(t, args.max_text)}")
            elif bt == "tool_use":
                name, inp = b.get("name", ""), b.get("input") or {}
                if name == "Bash":
                    label = inp.get("command", "")
                elif name in ("Edit", "Write", "MultiEdit", "Read"):
                    label = inp.get("file_path", "")
                    if name == "Edit":
                        label += f"  old={clip(inp.get('old_string', ''), 120)!r} new={clip(inp.get('new_string', ''), 160)!r}"
                    elif name == "Write":
                        label += f"  content={clip(inp.get('content', ''), 300)!r}"
                elif name == "Skill":
                    label = inp.get("skill", "") + " " + (inp.get("args") or "")
                elif name in ("Agent", "Task"):
                    label = clip(inp.get("prompt", ""), 300)
                else:
                    label = clip(json.dumps(inp), 200)
                is_mw = bool(MW_RX.search(label)) or (name == "Skill" and "meshwork" in label)
                tools[b.get("id")] = (name, is_mw, label)
                if args.mw and not is_mw:
                    continue
                print(f"    → {name}{side}: {clip(label, args.max_cmd)}")
    print(f"\n# title: {title}  turns: {turn}")


if __name__ == "__main__":
    main()
