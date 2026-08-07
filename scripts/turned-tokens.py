#!/usr/bin/env python3
"""Turned tokens — context re-submitted per turn, measured from transcripts.

Every API turn re-submits the whole accrued context; a late-session turn
that adds 2 tokens still pays for everything before it, at cache-read
rates when warm and cache-write rates when cold. This script sums, per
session, the actual per-request usage the API reported:

  turned    = cache_read + cache_creation + input   (context re-submitted)
  cached    = cache_read                            (~0.1x base price)
  uncached  = cache_creation + input                (1.25-2x / 1x base)

and prices it from the per-record model + the 5m/1h cache-write split.
It also attributes the tracker's share: tracker content resident in
context (classified as in admin-tokens.py, chars/4) is replayed on every
subsequent turn — resident_tracker summed over turns = tracker replay.

Usage:
  python3 scripts/turned-tokens.py <transcript.jsonl>...

Transcripts live in ~/.claude/projects/<encoded-repo-path>/*.jsonl.
Usage records are deduped by API message id; all records count toward
totals (sidechains included — they are real session cost); tracker
replay is attributed against main-chain turns only.

First run (2026-08-07, sazed): see README pilot table.
"""
import json
import re
import sys

PAT = re.compile(r"todo\.md|handoff|check-todo|meshwork", re.I)
ADMIN_TOOLS = {"TaskCreate", "TaskUpdate", "TaskList", "TaskGet", "TodoWrite"}

# $/MTok: (input, output, cache_read, write_5m, write_1h)
PRICE = {
    "opus": (5.0, 25.0, 0.50, 6.25, 10.0),
    "sonnet": (3.0, 15.0, 0.30, 3.75, 6.0),
    "haiku": (1.0, 5.0, 0.10, 1.25, 2.0),
}


def price_for(model):
    for k, v in PRICE.items():
        if k in (model or ""):
            return v
    return PRICE["opus"]


def content_chars(c):
    if isinstance(c, str):
        return len(c)
    if isinstance(c, list):
        return sum(len(b.get("text", "") or "") for b in c if isinstance(b, dict))
    return 0


def analyze(path):
    seen = set()
    turns = 0
    read = creation = uncached_in = out = 0
    cost = 0.0
    resident_admin = 0  # tokens of tracker content resident in context
    admin_replay = 0    # resident_admin summed over subsequent main-chain turns
    cold_rewrites = 0   # turns after the first with a large cache write
    cls = {}
    for line in open(path):
        try:
            e = json.loads(line)
        except json.JSONDecodeError:
            continue
        t = e.get("type")
        if t == "attachment":
            a = e.get("attachment", {})
            if a.get("type") == "hook_success" and "SessionStart" in (a.get("hookName") or ""):
                resident_admin += len(a.get("content", "") or "") // 4
            continue
        msg = e.get("message") or {}
        if t == "assistant":
            u = msg.get("usage")
            mid = msg.get("id")
            if u and mid and mid not in seen:
                seen.add(mid)
                turns += 1
                r = u.get("cache_read_input_tokens") or 0
                c5 = (u.get("cache_creation") or {}).get("ephemeral_5m_input_tokens") or 0
                c1 = (u.get("cache_creation") or {}).get("ephemeral_1h_input_tokens") or 0
                c = u.get("cache_creation_input_tokens") or (c5 + c1)
                if not (c5 or c1):
                    c1 = c  # no split reported; assume 1h (Claude Code default)
                i = u.get("input_tokens") or 0
                o = u.get("output_tokens") or 0
                read += r; creation += c; uncached_in += i; out += o
                p_in, p_out, p_read, p_w5, p_w1 = price_for(msg.get("model"))
                cost += (i * p_in + o * p_out + r * p_read + c5 * p_w5 + c1 * p_w1) / 1e6
                if turns > 1 and c > 20000:
                    cold_rewrites += 1
                if not e.get("isSidechain"):
                    admin_replay += min(resident_admin, r + c + i)
        content = msg.get("content")
        if not isinstance(content, list) or e.get("isSidechain"):
            continue
        for b in content:
            if not isinstance(b, dict):
                continue
            if b.get("type") == "tool_use":
                inp = b.get("input", {})
                target = " ".join(str(inp.get(k, "")) for k in ("command", "file_path"))
                is_admin = b.get("name") in ADMIN_TOOLS or bool(PAT.search(target))
                cls[b.get("id")] = is_admin
                if is_admin:
                    resident_admin += len(json.dumps(inp, ensure_ascii=False)) // 4
            elif b.get("type") == "tool_result" and cls.get(b.get("tool_use_id"), False):
                resident_admin += content_chars(b.get("content")) // 4
    return dict(turns=turns, read=read, creation=creation, uncached_in=uncached_in,
                out=out, cost=cost, admin_replay=admin_replay,
                cold_rewrites=cold_rewrites)


def fmt(n):
    return f"{n/1e6:.2f}M" if n >= 1e6 else f"{n//1000}K" if n >= 1000 else str(n)


def main():
    files = [a for a in sys.argv[1:] if not a.startswith("--")]
    if not files:
        sys.exit(__doc__)
    tot = None
    hdr = f"{'session':<14}{'turns':>6}{'turned':>10}{'cached':>10}{'uncached':>10}{'output':>8}{'replay/adm':>11}{'cold':>6}{'cost':>9}"
    print(hdr)
    for f in files:
        s = analyze(f)
        turned = s["read"] + s["creation"] + s["uncached_in"]
        name = f.rsplit("/", 1)[-1][:12]
        print(f"{name:<14}{s['turns']:>6}{fmt(turned):>10}{fmt(s['read']):>10}"
              f"{fmt(s['creation'] + s['uncached_in']):>10}{fmt(s['out']):>8}"
              f"{fmt(s['admin_replay']):>11}{s['cold_rewrites']:>6}{'$%.2f' % s['cost']:>9}")
        if tot is None:
            tot = {k: 0 for k in s}
        for k in s:
            tot[k] += s[k]
    if len(files) > 1:
        n = len(files)
        turned = tot["read"] + tot["creation"] + tot["uncached_in"]
        print(f"{'AVG/session':<14}{tot['turns']//n:>6}{fmt(turned//n):>10}{fmt(tot['read']//n):>10}"
              f"{fmt((tot['creation'] + tot['uncached_in'])//n):>10}{fmt(tot['out']//n):>8}"
              f"{fmt(tot['admin_replay']//n):>11}{tot['cold_rewrites']//n:>6}{'$%.2f' % (tot['cost']/n):>9}")


if __name__ == "__main__":
    main()
