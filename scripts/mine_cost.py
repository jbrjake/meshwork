#!/usr/bin/env python3
"""Price the findings — tokens per session, per task by category, main chain vs subagents.
Read-only; emits docs/cost-baseline.md.

    python3 scripts/mine_cost.py            # the tables, to stdout
    python3 scripts/mine_cost.py --write    # regenerate docs/cost-baseline.md
    python3 scripts/mine_cost.py --check    # exit 0 iff every denominator is non-zero and no
                                            # message was counted twice
    python3 scripts/mine_cost.py --json out.json

Sources, all read-only:
  1. session transcripts — ~/.claude/projects/<slug>/*.jsonl (CLAUDE_PROJECTS) for the nine
     registered repos and the code root, plus each session's subagent transcripts beside it
     (<slug>/<session>/subagents/agent-*.jsonl — the sidechain is a separate file, never inline):
     `message.usage` on every assistant message — input, output, cache-read and cache-creation
     tokens — counted ONCE per message id however many records the message spans
  2. the stores — per-repo `meshwork q` (never a portfolio verb: those autoprune sequence.md)
     for each task's category; MESHWORK_BIN, MESHWORK_CODE_ROOT, MESHWORK_REPOS as mine_rank.py

Attribution: on the main chain, tokens from `start <id>` until the next `close <id>` or the
next `start` belong to <id>; a subagent message belongs to whatever the main chain was on when
it appeared. Tokens before the first start or after a close are "between tasks". Categories
are the store's, folded to two levels (`engine/plan`), as mine_rank.py does.

Fresh tokens = input + cache-creation + output: what the API processed anew for the message.
Cache reads are reported beside them and in the hit rate (cache read ÷ all input), never
summed in — a 500k-context turn that is 98% cache hits is cheap; the same turn cold is not.
"""
import argparse
import bisect
import glob
import json
import os
import re
import statistics
import sys
from collections import Counter, defaultdict
from datetime import datetime

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from mine_sessions import (HEREDOC_STORE, MW_CMD, MW_REPOS, MW_WORD, PROJECTS_DIR, STORE_PATH,  # noqa: E402
                           TASK_ID, gid_of, human_text, is_heated, repo_of)
import mine_rank  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, "docs", "cost-baseline.md")
LONG_CALL_S = 300            # the DSL runner's RUN_TIMEOUT; a meshwork call past it hung on a build
RUNS_VERIFY = {"start", "close", "verify"}   # the verbs that can run a build; any other long call is a hold
# the sessions §2.1 of the field study names for the inbox eruptions, plus the five-eruption
# skill session of §2.3 — priced whole and from their first emphatic meshwork prompt onward
ERUPTIONS = ["72773264", "1b167145", "94b76cfa", "58b837fb", "f1040972", "51e60dbd"]


def stamp_s(ts):
    try:
        return datetime.fromisoformat(ts.replace("Z", "+00:00")).timestamp()
    except (ValueError, AttributeError):
        return None


def tally():
    return {"n": 0, "in": 0, "out": 0, "cr": 0, "cc": 0}


def add(t, u):
    t["n"] += 1
    t["in"] += u.get("input_tokens") or 0
    t["out"] += u.get("output_tokens") or 0
    t["cr"] += u.get("cache_read_input_tokens") or 0
    t["cc"] += u.get("cache_creation_input_tokens") or 0


def fresh(t):
    return t["in"] + t["cc"] + t["out"]


def hit_rate(t):
    d = t["cr"] + t["in"] + t["cc"]
    return t["cr"] / d if d else None


def price_file(path):
    s = {"session": os.path.basename(path)[:-6], "main": tally(), "side": tally(), "by_task": defaultdict(tally),
         "between": tally(), "skill": False, "mw_cmds": 0, "help": 0, "hand_edit_msgs": 0, "hand_edit_out": 0,
         "hand_edit_ctx": 0, "heat_mw_after": None, "heat_any_after": None, "long_calls": [], "n_user": 0,
         "subagents": 0, "start": "", "end": "", "seen": set(), "dups": 0, "usage_records": 0,
         "timeline": [(0.0, None)]}
    current = None                     # the task the main chain is on; timeline records each change
    call_at, call_verb = {}, {}        # tool_use id → stamp / verb, for the long-call measure
    hand_edit_mids = set()
    heat_started = False
    with open(path, encoding="utf-8", errors="replace") as f:
        for line in f:
            try:
                rec = json.loads(line)
            except Exception:
                continue
            typ, ts = rec.get("type"), rec.get("timestamp")
            if ts:
                s["start"] = s["start"] or ts
                s["end"] = ts
            if typ == "queue-operation" and rec.get("operation") == "enqueue":
                t = (rec.get("content") or "").strip()
                typ, rec = "user", {"type": "user", "message": {"content": t}}
            if typ == "user":
                t = human_text(rec)
                if t is not None:
                    s["n_user"] += 1
                    if s["n_user"] > 1 and is_heated(t):
                        if s["heat_any_after"] is None:
                            s["heat_any_after"] = tally()
                        if not heat_started and (MW_WORD.search(t) or TASK_ID.search(t)):
                            heat_started, s["heat_mw_after"] = True, tally()
                    continue
                for b in (rec.get("message") or {}).get("content") or []:
                    if isinstance(b, dict) and b.get("type") == "tool_result" and b.get("tool_use_id") in call_at:
                        t0, t1 = call_at.pop(b["tool_use_id"]), stamp_s(ts or "")
                        if t0 and t1 and t1 - t0 > LONG_CALL_S:
                            s["long_calls"].append((call_verb.get(b["tool_use_id"], "?"), (t1 - t0) / 60))
                continue
            if typ != "assistant" or rec.get("isApiErrorMessage"):
                continue
            msg = rec.get("message") or {}
            mid, u = msg.get("id"), msg.get("usage")
            side = bool(rec.get("isSidechain"))
            if mid and u:
                s["usage_records"] += 1
                if mid not in s["seen"]:
                    s["seen"].add(mid)
                    add(s["side" if side else "main"], u)
                    add(s["by_task"][current] if current else s["between"], u)
                    for key in ("heat_mw_after", "heat_any_after"):
                        if s[key] is not None:
                            add(s[key], u)
                else:
                    s["dups"] += 1
            for b in msg.get("content") or []:
                if not isinstance(b, dict) or b.get("type") != "tool_use":
                    continue
                name, inp, tid = b.get("name", ""), b.get("input") or {}, b.get("id")
                edit = False
                if name in ("Agent", "Task"):
                    s["subagents"] += 1
                elif name == "Skill" and "meshwork" in (inp.get("skill") or ""):
                    s["skill"] = True
                elif name in ("Edit", "Write", "MultiEdit") and STORE_PATH.search(inp.get("file_path") or ""):
                    edit = True
                elif name == "Bash":
                    cmd = inp.get("command") or ""
                    hits = MW_CMD.findall(cmd)
                    if hits:
                        s["mw_cmds"] += 1
                        s["help"] += 1 if "--help" in cmd else 0
                        ids = TASK_ID.findall(cmd)
                        call_at[tid], call_verb[tid] = stamp_s(ts or ""), hits[0][0]
                        if not side and ids:
                            for verb, _ in hits:
                                if verb == "start":
                                    current = gid_of(ids[0]) or f"?#{ids[0]}"
                                elif verb == "close" and current and current.endswith("#" + ids[0]):
                                    current = None
                                else:
                                    continue
                                s["timeline"].append((stamp_s(ts or "") or s["timeline"][-1][0], current))
                    if HEREDOC_STORE.search(cmd):
                        edit = True
                if edit and mid not in hand_edit_mids:
                    hand_edit_mids.add(mid)
                    s["hand_edit_msgs"] += 1
                    s["hand_edit_out"] += (u or {}).get("output_tokens") or 0
                    s["hand_edit_ctx"] += sum((u or {}).get(k) or 0 for k in ("input_tokens", "cache_read_input_tokens", "cache_creation_input_tokens"))
    return s


def price_subagent(path, s):
    """A subagent transcript beside its session: every message is sidechain, attributed to the
    task the main chain was on at the message's stamp."""
    times = [t for t, _ in s["timeline"]]
    with open(path, encoding="utf-8", errors="replace") as f:
        for line in f:
            try:
                rec = json.loads(line)
            except Exception:
                continue
            if rec.get("type") != "assistant" or rec.get("isApiErrorMessage"):
                continue
            msg = rec.get("message") or {}
            mid, u = msg.get("id"), msg.get("usage")
            if not (mid and u):
                continue
            s["usage_records"] += 1
            if mid in s["seen"]:
                s["dups"] += 1
                continue
            s["seen"].add(mid)
            add(s["side"], u)
            at = stamp_s(rec.get("timestamp") or "") or 0.0
            current = s["timeline"][bisect.bisect_right(times, at) - 1][1]
            add(s["by_task"][current] if current else s["between"], u)
            for key in ("heat_mw_after", "heat_any_after"):
                if s[key] is not None:
                    add(s[key], u)


def price_session(path):
    s = price_file(path)
    for sub in sorted(glob.glob(os.path.join(path[:-6], "subagents", "*.jsonl"))):
        price_subagent(sub, s)
    s["msgs"] = len(s["seen"])
    del s["seen"], s["timeline"]
    return s


def categories():
    """gid → two-level category, every registered store, per-repo q."""
    cat, missing = {}, []
    for repo in mine_rank.repos():
        rows = mine_rank.q(repo, "SELECT gid, category FROM tasks")
        if rows is None:
            missing.append(repo)
            continue
        for r in rows:
            cat[r["gid"]] = mine_rank.cat2(r.get("category"))
    return cat, missing


def med(xs):
    return statistics.median(xs) if xs else None


def p90(xs):
    return sorted(xs)[min(len(xs) - 1, int(0.9 * len(xs)))] if xs else None


def k(x, nd=0):
    return "—" if x is None else f"{x / 1000:.{nd}f}k"


def pc(x):
    return "—" if x is None else f"{100 * x:.0f}%"


def analyse(sessions, cat):
    r = {}
    used = [s for s in sessions if s["msgs"]]
    r["dataset"] = {"sessions": len(sessions), "with_usage": len(used), "dups": sum(s["dups"] for s in sessions),
                    "first": min((s["start"] for s in used if s["start"]), default=None),
                    "last": max((s["end"] for s in used if s["end"]), default=None),
                    "messages": sum(s["msgs"] for s in used)}
    # --- per session, by repo ---------------------------------------------------------------
    by_repo = defaultdict(list)
    for s in used:
        by_repo[s["repo"]].append(s)
    rows = []
    for repo, ss in sorted(by_repo.items(), key=lambda kv: -sum(fresh(x["main"]) + fresh(x["side"]) for x in kv[1])):
        tot = [fresh(s["main"]) + fresh(s["side"]) for s in ss]
        rows.append({"repo": repo, "n": len(ss), "fresh_med": med(tot), "fresh_p90": p90(tot),
                     "fresh_sum": sum(tot), "out_med": med([s["main"]["out"] + s["side"]["out"] for s in ss]),
                     "msgs_med": med([s["msgs"] for s in ss]),
                     "hit_med": med([h for h in (hit_rate(s["main"]) for s in ss) if h is not None]),
                     "side_share": sum(fresh(s["side"]) for s in ss) / sum(tot) if sum(tot) else 0,
                     "fanout": sum(1 for s in ss if s["side"]["n"])})
    tot = [fresh(s["main"]) + fresh(s["side"]) for s in used]
    rows.append({"repo": "all", "n": len(used), "fresh_med": med(tot), "fresh_p90": p90(tot), "fresh_sum": sum(tot),
                 "out_med": med([s["main"]["out"] + s["side"]["out"] for s in used]), "msgs_med": med([s["msgs"] for s in used]),
                 "hit_med": med([h for h in (hit_rate(s["main"]) for s in used) if h is not None]),
                 "side_share": sum(fresh(s["side"]) for s in used) / sum(tot) if sum(tot) else 0,
                 "fanout": sum(1 for s in used if s["side"]["n"])})
    r["by_repo"] = rows
    # --- cache -----------------------------------------------------------------------------------
    hits = sorted(h for h in (hit_rate(s["main"]) for s in used) if h is not None)
    cold = sorted(((hit_rate(s["main"]), s) for s in used if hit_rate(s["main"]) is not None and hit_rate(s["main"]) < 0.5),
                  key=lambda x: x[0])
    r["cache"] = {"n": len(hits), "p10": hits[int(0.1 * len(hits))] if hits else None, "med": med(hits),
                  "p90": p90(hits), "below_half": len(cold),
                  "cold": [{"repo": s["repo"], "session": s["session"][:8], "hit": h, "msgs": s["msgs"],
                            "fresh": fresh(s["main"])} for h, s in cold],
                  "corpus": hit_rate({"cr": sum(s["main"]["cr"] for s in used), "in": sum(s["main"]["in"] for s in used),
                                      "cc": sum(s["main"]["cc"] for s in used)})}
    # --- main chain vs subagents ---------------------------------------------------------------
    fan = [s for s in used if s["side"]["n"]]
    r["fanout"] = {"sessions": len(fan), "of": len(used), "side_fresh": sum(fresh(s["side"]) for s in used),
                   "main_fresh": sum(fresh(s["main"]) for s in used),
                   "side_share_in_fanout": (sum(fresh(s["side"]) for s in fan) / sum(fresh(s["side"]) + fresh(s["main"]) for s in fan)) if fan else None,
                   "fan_fresh_med": med([fresh(s["main"]) + fresh(s["side"]) for s in fan]),
                   "nofan_fresh_med": med([fresh(s["main"]) for s in used if not s["side"]["n"]]),
                   "agents_med": med([s["subagents"] for s in fan]), "agents_p90": p90([s["subagents"] for s in fan])}
    # --- per task, by category -------------------------------------------------------------------
    per_task = defaultdict(tally)
    for s in used:
        for gid, t in s["by_task"].items():
            pt = per_task[gid]
            for key in ("n", "in", "out", "cr", "cc"):
                pt[key] += t[key]
    by_cat = defaultdict(list)
    unknown = 0
    for gid, t in per_task.items():
        c = cat.get(gid)
        if c is None:
            unknown += 1
            continue
        by_cat[c].append(fresh(t))
    cats = [{"category": c, "n": len(v), "med": med(v), "p90": p90(v), "sum": sum(v)}
            for c, v in by_cat.items()]
    cats.sort(key=lambda x: -x["n"])
    big = [c for c in cats if c["n"] >= 7]
    meds = [c["med"] for c in big if c["med"]]
    r["tasks"] = {"attributed": len(per_task), "unknown_gid": unknown, "fresh_med": med([fresh(t) for t in per_task.values()]),
                  "fresh_p90": p90([fresh(t) for t in per_task.values()]),
                  "between_fresh": sum(fresh(s["between"]) for s in used),
                  "attributed_fresh": sum(fresh(t) for t in per_task.values()),
                  "cats": cats, "n7": len(big), "spread": (max(meds) / min(meds)) if len(meds) >= 2 and min(meds) else None,
                  "per_task": {gid: {"fresh": fresh(t), "category": cat.get(gid)} for gid, t in per_task.items()},
                  "spread_lo": min(big, key=lambda c: c["med"])["category"] if big else None,
                  "spread_hi": max(big, key=lambda c: c["med"])["category"] if big else None}
    # --- skill loaded vs not ---------------------------------------------------------------------
    def cut(ss):
        sk, no = [s for s in ss if s["skill"]], [s for s in ss if not s["skill"]]
        return {"skill": summ(sk), "no_skill": summ(no)}

    def summ(ss):
        return {"n": len(ss), "fresh_med": med([fresh(s["main"]) + fresh(s["side"]) for s in ss]),
                "per_call_med": med([(fresh(s["main"]) + fresh(s["side"])) / s["mw_cmds"] for s in ss if s["mw_cmds"]]),
                "hand_med": med([s["hand_edit_msgs"] for s in ss]), "help_med": med([s["help"] for s in ss]),
                "hand_sum": sum(s["hand_edit_msgs"] for s in ss), "help_sum": sum(s["help"] for s in ss),
                "calls_med": med([s["mw_cmds"] for s in ss])}
    r["skill"] = {"any": cut([s for s in used if s["mw_cmds"]]), "heavy": cut([s for s in used if s["mw_cmds"] >= 10])}
    # --- hand-edits ------------------------------------------------------------------------------
    hs = [s for s in used if s["hand_edit_msgs"]]
    r["hand"] = {"msgs": sum(s["hand_edit_msgs"] for s in used), "sessions": len(hs),
                 "out": sum(s["hand_edit_out"] for s in used), "ctx": sum(s["hand_edit_ctx"] for s in used),
                 "out_share": sum(s["hand_edit_out"] for s in used) / sum(s["main"]["out"] + s["side"]["out"] for s in used)}
    # --- long calls ------------------------------------------------------------------------------
    lc = [(s["repo"], v, m) for s in used for v, m in s["long_calls"]]
    runs = [(rp, v, m) for rp, v, m in lc if v in RUNS_VERIFY]
    holds = [(rp, v, m) for rp, v, m in lc if v not in RUNS_VERIFY]
    r["long"] = {"n": len(lc), "hours": sum(m for _, _, m in lc) / 60, "sessions": len({s["session"] for s in used if s["long_calls"]}),
                 "by_verb": Counter(v for _, v, _ in lc).most_common(), "max_min": max((m for _, _, m in lc), default=0),
                 "runs": len(runs), "runs_h": sum(m for _, _, m in runs) / 60,
                 "runs_by_repo": Counter(rp for rp, _, _ in runs).most_common(),
                 "holds": len(holds), "holds_h": sum(m for _, _, m in holds) / 60,
                 "holds_by_verb": Counter(v for _, v, _ in holds).most_common(6)}
    # --- the eruption sessions -------------------------------------------------------------------
    er = []
    for s in used:
        if s["session"][:8] in ERUPTIONS:
            er.append({"repo": s["repo"], "session": s["session"][:8], "fresh": fresh(s["main"]) + fresh(s["side"]),
                       "out": s["main"]["out"] + s["side"]["out"], "msgs": s["msgs"],
                       "after_heat": fresh(s["heat_any_after"]) if s["heat_any_after"] else None,
                       "after_heat_msgs": s["heat_any_after"]["n"] if s["heat_any_after"] else None})
    er.sort(key=lambda e: -e["fresh"])
    r["eruptions"] = er
    heat = [s for s in used if s["heat_mw_after"]]
    r["heat"] = {"sessions": len(heat), "after_fresh": sum(fresh(s["heat_mw_after"]) for s in heat),
                 "after_share_med": med([fresh(s["heat_mw_after"]) / (fresh(s["main"]) + fresh(s["side"])) for s in heat
                                         if fresh(s["main"]) + fresh(s["side"])])}
    return r


def render(r, ts):
    d, o = r["dataset"], []
    w = o.append
    w("# The cost baseline\n")
    w("Tokens per session, per task by category, and main chain against subagents, mined from the\n"
      "session transcripts of the nine registered repos and the code root. The setup-cost matrix's\n"
      f"sibling. Data through **{d['last'][:16].replace('T', ' ')}Z**; the corpus is live while the\n"
      "miner runs, so a re-run drifts by units. Read-only miner: `python3 scripts/mine_cost.py --write`\n"
      "regenerates this file; `--check` is its self-test.\n")
    w("## Headlines (each number's denominator in its section below)\n")
    a = r["by_repo"][-1]
    t, f, c = r["tasks"], r["fanout"], r["cache"]
    w(f"- A session processes a median **{k(a['fresh_med'])} fresh tokens** (p90 {k(a['fresh_p90'])}) over a median "
      f"{a['msgs_med']:.0f} API messages, at a median cache hit rate of **{pc(a['hit_med'])}** (n={a['n']} sessions).")
    w(f"- **{pc(f['side_fresh'] / (f['side_fresh'] + f['main_fresh']))} of all fresh tokens are spent on subagents**, in "
      f"{f['sessions']} of {f['of']} sessions; a session that fans out costs a median {k(f['fan_fresh_med'])} against "
      f"{k(f['nofan_fresh_med'])} for one that does not.")
    if t["spread"]:
        w(f"- Per task, by category (n ≥ 7 tasks): medians span **{t['spread']:.1f}×**, from `{t['spread_lo']}` to "
          f"`{t['spread_hi']}` — the prioritization ladder's rung-0 question (*does cost vary ≥ 3× across categories?*) "
          f"is answered **{'yes' if t['spread'] >= 3 else 'no'}** on {t['n7']} categories.")
    s = r["skill"]["heavy"]
    w(f"- Sessions that loaded the skill (≥ 10 meshwork calls) spend a median **{k(s['skill']['per_call_med'], 1)} fresh "
      f"tokens per meshwork call** against {k(s['no_skill']['per_call_med'], 1)} without it "
      f"(n={s['skill']['n']}/{s['no_skill']['n']}); hand-edits per session do not move (median "
      f"{s['skill']['hand_med']:.0f} both ways).")
    h = r["hand"]
    w(f"- Hand-edits of task files cost **{h['msgs']} API turns** in {h['sessions']} sessions — {k(h['out'])} output "
      f"tokens ({pc(h['out_share'])} of all output) on turns carrying {h['ctx'] / 1e6:.0f}M tokens of context, "
      "nearly all of it cache reads.")
    lg = r["long"]
    w(f"- Meshwork calls whose result took more than {LONG_CALL_S // 60} minutes: **{lg['n']}**, {lg['hours']:.1f} h of wall "
      f"time in {lg['sessions']} sessions (longest {lg['max_min']:.0f} min). {lg['runs']} of them were `start`/`close`/`verify` "
      f"({lg['runs_h']:.1f} h — a build, or an approval hold); the other {lg['holds']} ({lg['holds_h']:.1f} h) were verbs that "
      f"run nothing ({dict(lg['holds_by_verb'])}) and can only be the owner holding an approval: §1.6's wait, at "
      "the permission prompt.\n")
    w("## Method\n")
    w("- **Session** = one transcript file. **Message** = one assistant API response, identified by `message.id`; "
      "a response streams as several records that repeat the same `usage`, so it is counted once "
      f"({d['dups']} duplicate records skipped). Synthetic API-error records carry zero usage and are skipped.")
    w("- **Fresh tokens** = `input_tokens + cache_creation_input_tokens + output_tokens`: what the API processed anew. "
      "**Cache hit rate** = `cache_read ÷ (cache_read + input + cache_creation)`, main chain. Cache reads are never "
      "summed into fresh.")
    w("- **Main chain vs subagents** = the session's transcript against the `subagents/agent-*.jsonl` files beside it "
      "(every record there is `isSidechain`). A subagent message is attributed to the task the main chain was on "
      "at the message's stamp.")
    w("- **Task attribution** = tokens from a main-chain `start <id>` until that id's `close` or the next `start`; "
      "before the first start or after a close they are *between tasks*. Categories are the store's, two levels, "
      "per-repo `q` over every store including archives.")
    w("- **Hand-edit turn** = a message carrying an Edit/Write on `docs/meshwork/*.md` or a heredoc/sed/perl into one. "
      "**Long call** = a meshwork Bash call whose `tool_result` landed more than five minutes after the call.")
    w("- **After the first emphatic meshwork prompt** = usage from the first prompt after turn 1 that is emphatic "
      "(field study §1.1's definition) and names meshwork or a task id, to the end of the session.\n")
    w("## Dataset (the denominators)\n")
    w(f"{d['with_usage']} of {d['sessions']} transcripts carry usage, {d['first'][:10]} → {d['last'][:10]}, "
      f"{d['messages']:,} API messages.\n")
    w("| repo | sessions | med fresh / session | p90 | total fresh | med output | med messages | med cache hit | subagent share | sessions with subagents |")
    w("|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|")
    for x in r["by_repo"]:
        w(f"| {'**' + x['repo'] + '**' if x['repo'] == 'all' else x['repo']} | {x['n']} | {k(x['fresh_med'])} | {k(x['fresh_p90'])} | "
          f"{x['fresh_sum'] / 1e6:.1f}M | {k(x['out_med'])} | {x['msgs_med']:.0f} | {pc(x['hit_med'])} | {pc(x['side_share'])} | {x['fanout']} |")
    w("\n## Cache\n")
    w(f"Per-session main-chain hit rate (n={c['n']}): p10 {pc(c['p10'])}, median {pc(c['med'])}, p90 {pc(c['p90'])}; "
      f"corpus-wide {pc(c['corpus'])}. Caching is near-universal, so the fresh-token figures above are the cost that "
      f"varies. The {c['below_half']} sessions below 50% are short: "
      + ", ".join(f"{x['repo']}/{x['session']} ({pc(x['hit'])}, {x['msgs']} msgs)" for x in c["cold"]) + ".\n")
    w("## Main chain vs subagents\n")
    w(f"| | value |\n|---|---|\n| sessions that spawned subagents | {f['sessions']} of {f['of']} |\n"
      f"| fresh tokens on subagents, corpus | {f['side_fresh'] / 1e6:.1f}M of {(f['side_fresh'] + f['main_fresh']) / 1e6:.1f}M "
      f"({pc(f['side_fresh'] / (f['side_fresh'] + f['main_fresh']))}) |\n"
      f"| subagent share inside fan-out sessions | {pc(f['side_share_in_fanout'])} |\n"
      f"| median fresh, fan-out vs single-chain sessions | {k(f['fan_fresh_med'])} vs {k(f['nofan_fresh_med'])} |\n"
      f"| subagents per fan-out session | median {f['agents_med']:.0f}, p90 {f['agents_p90']:.0f} |\n")
    w("The field study's own method — 39 sessions read by analyst subagents from the portfolio and code-root "
      "directories — is inside these numbers, which is why those two repos carry the largest subagent shares.\n")
    w("## Per task, by category\n")
    w(f"{t['attributed']} tasks received attributed tokens ({t['attributed_fresh'] / 1e6:.1f}M fresh; "
      f"{t['between_fresh'] / 1e6:.1f}M more fell between tasks); {t['unknown_gid']} attributed ids resolve to no store. "
      f"Per task: median {k(t['fresh_med'])}, p90 {k(t['fresh_p90'])}. Categories with n ≥ 7:\n")
    w("| category | tasks | med fresh / task | p90 | total |\n|---|---:|---:|---:|---:|")
    for x in t["cats"]:
        if x["n"] >= 7:
            w(f"| {x['category']} | {x['n']} | {k(x['med'])} | {k(x['p90'])} | {x['sum'] / 1e6:.1f}M |")
    if t["spread"]:
        w(f"\nSpread of medians across those {t['n7']} categories: **{t['spread']:.1f}×** (`{t['spread_lo']}` → "
          f"`{t['spread_hi']}`). Rung 0's threshold is 3×.\n")
    w("## Skill loaded vs not\n")
    w("| sessions | n | med fresh / session | med fresh / meshwork call | med meshwork calls | med hand-edits | med --help |\n|---|---:|---:|---:|---:|---:|---:|")
    for label, cutname in (("any meshwork call, skill loaded", "any"), ("≥ 10 calls, skill loaded", "heavy")):
        for side, name in (("skill", label), ("no_skill", label.replace("skill loaded", "not loaded"))):
            x = r["skill"][cutname][side]
            w(f"| {name} | {x['n']} | {k(x['fresh_med'])} | {k(x['per_call_med'], 1)} | {x['calls_med']:.0f} | {x['hand_med']:.0f} | {x['help_med']:.0f} |")
    w("\n## The findings, priced\n")
    w("| finding (field study) | price |\n|---|---|")
    w(f"| 723 hand-edits of task files (§2.3) | {h['msgs']} API turns in {h['sessions']} sessions; {k(h['out'])} output tokens "
      f"({pc(h['out_share'])} of all output), {h['ctx'] / 1e6:.0f}M tokens of context re-sent |")
    w(f"| the unscoped verify — an 18-minute compile against a 5-minute timeout (§2.5.2–3) | {lg['runs']} `start`/`close`/"
      f"`verify` calls past {LONG_CALL_S // 60} min, {lg['runs_h']:.1f} h of wall time, by repo {dict(lg['runs_by_repo'])}; "
      f"plus {lg['holds']} calls on verbs that run nothing ({lg['holds_h']:.1f} h) — approval holds |")
    w(f"| the sixth-time eruptions (§2.1, §2.3) | " + "; ".join(
        f"{e['repo']}/{e['session']} {k(e['fresh'])} fresh" + (f", {k(e['after_heat'])} of it after the first emphatic prompt" if e['after_heat'] else "")
        for e in r["eruptions"]) + " |")
    ht = r["heat"]
    w(f"| every session with an emphatic meshwork prompt | {ht['sessions']} sessions spent {ht['after_fresh'] / 1e6:.1f}M fresh tokens "
      f"after their first one — a median {pc(ht['after_share_med'])} of the session |")
    w(f"| 61% of sessions never load the skill (§1.2) | per meshwork call, {k(s['no_skill']['per_call_med'], 1)} fresh without the "
      f"skill vs {k(s['skill']['per_call_med'], 1)} with it (≥ 10-call sessions, n={s['no_skill']['n']}/{s['skill']['n']}) |")
    w(f"| fan-out (ask A8) | {pc(f['side_fresh'] / (f['side_fresh'] + f['main_fresh']))} of the corpus's fresh tokens; "
      f"{f['sessions']} sessions |")
    w(f"| cache hit rate (ask A4) | median {pc(c['med'])} per session, p10 {pc(c['p10'])}; {c['below_half']} sessions under half — "
      "not where the cost varies |\n")
    w("## Caveats\n")
    w("- Transcripts exist only on this machine and are prunable; the meshwork repo's sessions dogfood the CLI against "
      "fixture stores, so its attributed ids include throwaway tasks.")
    w("- Half of all closed work was never `start`ed (field study §1.2), so per-task attribution covers the claimed "
      "half; unclaimed work is in *between tasks*. A session that starts a task and never closes it attributes "
      "everything after the start to it.")
    w("- Token counts are the API's; no price list is applied. Cache reads are billed but are not fresh work, which "
      "is why they are reported apart.")
    w("- Verb and edit detection is regex over commands, as in the field study; a heredoc that mentions a verb in "
      "prose can miscount. Hand-edits and meshwork calls made by subagents are not counted (the field study's "
      "scripts read the session transcript only); their tokens are.")
    w("- A long call's duration includes any approval hold in front of it; the split by verb is the only separation "
      "the transcript offers.")
    return "\n".join(o) + "\n"


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--write", action="store_true")
    ap.add_argument("--check", action="store_true")
    ap.add_argument("--json")
    args = ap.parse_args()
    sessions = []
    for d in sorted(glob.glob(os.path.join(PROJECTS_DIR, "*"))):
        repo = repo_of(os.path.basename(d))
        if repo not in MW_REPOS:
            continue
        for p in glob.glob(os.path.join(d, "*.jsonl")):
            s = price_session(p)
            s["repo"] = repo
            sessions.append(s)
    cat, missing = categories()
    r = analyse(sessions, cat)
    r["stores_missing"] = missing
    if args.check:
        bad = []
        if not r["dataset"]["with_usage"]:
            bad.append("no session carries usage")
        if r["tasks"]["n7"] == 0:
            bad.append("no category reaches n ≥ 7")
        if missing:
            bad.append(f"stores unreadable: {missing}")
        if r["fanout"]["sessions"] == 0:
            bad.append("no fan-out session")
        # every message counted once: records carrying usage = messages counted + duplicates skipped
        if any(s["usage_records"] != s["msgs"] + s["dups"] for s in sessions):
            bad.append("a message was counted twice or dropped")
        print("check: " + ("ok" if not bad else "FAIL — " + "; ".join(bad)))
        sys.exit(1 if bad else 0)
    text = render(r, datetime.now())
    if args.json:
        with open(args.json, "w") as f:
            json.dump(r, f, indent=1, default=str)
    if args.write:
        with open(OUT, "w") as f:
            f.write(text)
        print(f"wrote {OUT}")
    else:
        sys.stdout.write(text)


if __name__ == "__main__":
    main()
