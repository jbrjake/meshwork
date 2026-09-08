#!/usr/bin/env python3
"""Corpus-wide meshwork usage telemetry from session transcripts. Read-only.

    python3 scripts/mine_sessions.py --json scores.json --events events.jsonl
    python3 scripts/mine_telemetry.py scores.json events.jsonl [--json out.json]

Answers, with denominators: which verbs agents use and guess, which errors recur (normalized
message text), what they ask --help about, whether sessions start on prime's `next →`, whether
asks addressed to the repo get touched, how close refusals are resolved (fix the code, edit the
verify, hand-edit, waive), how often the store is read with grep/find instead of `search`,
cross-repo hops, prime size, bookkeeping share of tool calls, and the weekly trend.
"""
import argparse
import json
import re
import statistics
from collections import Counter, defaultdict
from datetime import datetime

ID = re.compile(r"\b[a-z]{2}-[a-z0-9]{4,7}\b")
PATH = re.compile(r"(?:/[\w.@-]+){2,}")
NUM = re.compile(r"\b\d+\b")
HOP = re.compile(r"cd \.\./([\w-]+)")


def norm(line):
    line = ID.sub("ID", line)
    line = PATH.sub("PATH", line)
    line = NUM.sub("N", line)
    return line[:110]


def pct(a, b):
    return f"{a}/{b} ({100.0 * a / b:.0f}%)" if b else f"{a}/0"


def load_event(line):
    """One envelope record → the envelope keys plus its `detail` payload, flat, for this script's
    own use; `id` is the bare id of `gid`, `verbs` is [] for non-call kinds."""
    e = json.loads(line)
    d = e.pop("detail") or {}
    ids = d.get("ids") or []
    return {**e, **d, "id": ids[0] if ids else "", "verbs": d.get("verbs") or []}


def wait_table(rows):
    """What the owner costs the agents: the waits after end_turn, with denominators. Reads the
    per-session `waits` and `unanswered_stops` mine_sessions.py computes (its header defines both)."""
    waits = [(r, w) for r in rows for w in r.get("waits", [])]
    stops = Counter()
    for r in rows:
        stops.update(r.get("stops", {}))
    prompts = sum(r["n_user"] for r in rows)
    unanswered = sum(r.get("unanswered_stops", 0) for r in rows)
    continued = sum(r.get("continued_stops", 0) for r in rows)
    days = sorted({r["start"][:10] for r in rows if r["start"]} | {r["end"][:10] for r in rows if r["end"]})
    span = (datetime.fromisoformat(days[-1]) - datetime.fromisoformat(days[0])).days + 1 if days else 0
    o = {"sessions": len(rows), "with_waits": len({r["session"] for r, _ in waits}), "waits": len(waits),
         "unanswered_stops": unanswered, "continued_stops": continued, "end_turn_stops": stops.get("end_turn", 0),
         "tool_use_stops": stops.get("tool_use", 0), "prompts": prompts, "span_days": span,
         "first_day": days[0] if days else None, "last_day": days[-1] if days else None}
    print(f"\nwaits after end_turn — {len(waits)} in {o['with_waits']} of {len(rows)} sessions, "
          f"{days[0] if days else '?'} → {days[-1] if days else '?'} ({span} d); stops: end_turn {o['end_turn_stops']} "
          f"(answered by a prompt {len(waits)}, the agent went on by itself {continued}, no following prompt "
          f"{unanswered}), tool_use {o['tool_use_stops']}; human prompts {prompts}")
    if not waits:
        return o
    mins = sorted(w["wait_min"] for _, w in waits)
    typed = sorted(w["wait_min"] for _, w in waits if not w["queued"])
    total = sum(mins)
    pq = lambda xs, p: xs[min(len(xs) - 1, int(p * len(xs)))]
    queued = sum(1 for _, w in waits if w["queued"])
    over_day = [m for m in mins if m > 24 * 60]
    o.update({"queued_zero": queued, "p50": pq(mins, .5), "p75": pq(mins, .75), "p90": pq(mins, .9),
              "p99": pq(mins, .99), "max": mins[-1], "typed_p50": pq(typed, .5) if typed else None,
              "total_h": total / 60, "share_over_1h": sum(m for m in mins if m > 60) / total if total else 0,
              "over_24h": len(over_day), "over_24h_h": sum(over_day) / 60})
    print(f"  queued (typed mid-turn, zero wait): {queued}; median {o['p50']:.1f} min ({o['typed_p50']:.1f} over the "
          f"{len(typed)} typed after the stop), p75 {o['p75']:.1f}, p90 {o['p90']:.1f}, p99 {o['p99'] / 60:.1f} h, "
          f"max {o['max'] / 60:.1f} h; total idle {o['total_h']:.0f} h, {100 * o['share_over_1h']:.0f}% of it in "
          f"waits > 1 h; waits > 24 h (a session resumed days later): {len(over_day)}, {o['over_24h_h']:.0f} h")
    for label, floor in (("ge15", 15), ("ge60", 60)):
        ws = [w for _, w in waits if w["wait_min"] >= floor]
        idle = sum(w["wait_min"] for w in ws) / 60
        known = [w for w in ws if w.get("live_at_stop") is not None]
        live = sum(1 for w in known if w["live_at_stop"])
        live_h = sum(w.get("live_min") or 0 for w in ws) / 60
        owner = [w.get("owner_min") or 0 for w in ws]
        owner_h = sum(owner) / 60
        med_owner = statistics.median(owner) if owner else 0
        over30 = sum(1 for m in owner if m >= 30)
        wdays = {w["at"][:10] for w in ws}
        o[label] = {"n": len(ws), "idle_h": idle, "live_at_stop": live, "live_h": live_h, "owner_h": owner_h,
                    "owner_min_median": med_owner, "owner_ge30": over30, "days": len(wdays)}
        print(f"  waits ≥ {floor} min: {len(ws)}, {idle:.0f} h idle; began while another session was live: "
              f"{pct(live, len(known))}; minutes another transcript was live inside them {live_h:.0f} h; minutes the "
              f"owner was attending another transcript {owner_h:.0f} h (median {med_owner:.0f} min per wait; "
              f"≥ 30 min in {over30}); on {len(wdays)} of {span} days")
    per_repo = Counter()
    for r, w in waits:
        per_repo[r["repo"]] += w["wait_min"] / 60
    o["idle_h_by_repo"] = dict(per_repo)
    print("  idle hours by repo:", ", ".join(f"{k} {v:.0f}" for k, v in per_repo.most_common()))
    after = Counter()
    for r in rows:
        after.update(r.get("prompt_after", {}))
    o["prompt_after"] = dict(after)
    print("  what a prompt after the first followed — the agent had stopped (end_turn) or was interrupted "
          "mid-call (tool_use):", after.most_common())
    settled = [r for r in rows if r["end"] and r["end"][:10] < (days[-1] if days else "")]
    o["tool_uses"] = sum(r.get("tool_uses", 0) for r in settled)
    o["orphan_tool_uses"] = sum(r.get("orphan_tool_uses", 0) for r in settled)
    o["denials_all_tools"] = sum(r.get("denials_all", 0) for r in rows)
    print(f"  tool_use blocks in transcripts not touched today: {o['tool_uses']}, never answered by a tool_result: "
          f"{o['orphan_tool_uses']}; tool calls the owner declined, all tools: {o['denials_all_tools']}")
    return o


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("scores")
    ap.add_argument("events")
    ap.add_argument("--json")
    args = ap.parse_args()
    rows = json.load(open(args.scores))
    by_sid = {r["session"]: r for r in rows}
    events = [load_event(l) for l in open(args.events)]
    calls = [e for e in events if e["kind"] == "call"]
    per_session = defaultdict(list)
    for e in events:
        per_session[e["session"]].append(e)
    out = {}

    used = [r for r in rows if r["mw_cmds"]]
    dates = sorted(r["start"][:10] for r in used if r["start"])
    print(f"corpus: {len(rows)} sessions in meshwork repos, {len(used)} used meshwork, "
          f"{dates[0] if dates else '?'} → {dates[-1] if dates else '?'}")
    out["corpus"] = {"sessions": len(rows), "used": len(used), "first": dates[0] if dates else None,
                     "last": dates[-1] if dates else None}

    # ---- verbs -------------------------------------------------------------------------------
    verbs, guessed, helps = Counter(), Counter(), Counter()
    for r in rows:
        verbs.update(r["verbs"])
        guessed.update(r["guessed"])
    for e in calls:
        if e["help"]:
            for v in e["verbs"]:
                helps[v] += 1
    print("\nverbs:", verbs.most_common(30))
    print("guessed verbs:", guessed.most_common(25))
    print("--help probes by verb:", helps.most_common(20), " total", sum(helps.values()))
    out["verbs"], out["guessed"], out["help"] = dict(verbs), dict(guessed), dict(helps)

    # ---- errors ------------------------------------------------------------------------------
    kinds, lines = Counter(), Counter()
    for e in calls:
        if e.get("err"):
            kinds[e["err"]] += 1
            lines[(e["err"], norm(e.get("err_line", "")))] += 1
    total_calls = len(calls)
    print(f"\nerrors: {sum(kinds.values())} of {total_calls} meshwork calls; kinds:", kinds.most_common())
    print("top normalized error lines:")
    for (k, l), n in lines.most_common(28):
        print(f"  {n:4d}  [{k}] {l}")
    out["err_kinds"] = dict(kinds)
    out["err_lines"] = [{"kind": k, "line": l, "n": n} for (k, l), n in lines.most_common(40)]

    # ---- close refusals → what happened next -------------------------------------------------
    res = Counter()
    sub = Counter()
    waives = sum(1 for e in calls if e["waive"])
    set_verify_total = sum(1 for e in calls if e["set_verify"])

    def refusal_kind(line):
        if "unapproved" in line:
            return "unapproved-verify"
        if "malformed" in line:
            return "malformed-verify"
        if "no verify" in line:
            return "no-verify"
        if "stays" in line:
            return "verify-failed"
        return "other"

    for sid, evs in per_session.items():
        for i, e in enumerate(evs):
            if e["kind"] != "call" or e.get("err") != "close-refused" or not e["id"]:
                continue
            tid = e["id"]
            res["refusals"] += 1
            rk = refusal_kind(e.get("err_line", ""))
            sub[rk] += 1
            modified = None
            outcome = "never-closed"
            for f in evs[i + 1:]:
                if f["kind"] not in ("call", "hand-edit") or (tid not in f.get("ids", []) and f["id"] != tid):
                    continue
                if f.get("set_verify") or f["kind"] == "hand-edit":
                    modified = modified or ("set-verify" if f.get("set_verify") else "hand-edit")
                if "close" in f["verbs"]:
                    if f["waive"]:
                        outcome = "waived"
                        break
                    if not f.get("err"):
                        outcome = "closed-after-" + (modified or "no-store-edit")
                        break
            res[outcome] += 1
            sub[f"{rk} → {outcome}"] += 1
    print(f"\nclose refusals and their resolution: {dict(res)}   --waive uses: {waives}   set --verify uses: {set_verify_total}")
    print("  by refusal kind:", [(k, v) for k, v in sub.most_common() if "→" not in k])
    print("  kind → outcome:", [(k, v) for k, v in sub.most_common() if "→" in k])
    out["close_refusals"] = dict(res)
    out["close_refusal_kinds"] = dict(sub)
    out["waives"], out["set_verify"] = waives, set_verify_total

    # ---- prime adherence ---------------------------------------------------------------------
    adher = Counter()
    asks = Counter()
    for r in rows:
        evs = per_session.get(r["session"], [])
        if not r.get("prime_next"):
            continue
        nxt = r["prime_next"].split("#")[-1]
        own = nxt[:2]
        first_start = next((e["id"] for e in evs if e["kind"] == "call" and "start" in e["verbs"] and e["id"]), None)
        if r["mw_cmds"] == 0:
            adher["prime-shown-no-meshwork-call"] += 1
        elif first_start is None:
            adher["no-start-at-all"] += 1
        elif first_start == nxt:
            adher["started-on-next"] += 1
        else:
            adher["started-elsewhere"] += 1
        if r.get("prime_addressed", 0) > 0:
            asks["sessions-with-asks-in-prime"] += 1
            touched = any((e["id"] and e["id"][:2] != own) or e.get("hop") or "answers" in e["note"]
                          for e in evs if e["kind"] in ("call", "hand-edit"))
            asks["touched-a-foreign-id-or-hopped"] += 1 if touched else 0
    print(f"\nprime adherence (sessions where the hook injected prime): {dict(adher)}")
    print(f"asks addressed to the repo: {dict(asks)}")
    out["prime_adherence"], out["asks"] = dict(adher), dict(asks)

    sizes = sorted(r["prime_bytes"] for r in rows if r.get("prime_bytes"))
    if sizes:
        q = lambda p: sizes[min(len(sizes) - 1, int(p * len(sizes)))]
        print(f"prime bytes: n={len(sizes)} p50={q(.5)} p90={q(.9)} max={sizes[-1]} over-6144={sum(s > 6144 for s in sizes)}")
        out["prime_bytes"] = {"n": len(sizes), "p50": q(.5), "p90": q(.9), "max": sizes[-1], "over_6144": sum(s > 6144 for s in sizes)}

    # ---- store access outside the CLI --------------------------------------------------------
    shell_reads = [e for e in events if e["kind"] == "shell-read"]
    hand = [e for e in events if e["kind"] == "hand-edit"]
    searches = sum(1 for e in calls if "search" in e["verbs"])
    tool_rx = re.compile(r"\b(grep|rg|find|ls|cat|head|sed|awk)\b[^\n|]*docs/meshwork")
    by_tool = Counter(m.group(1) for e in shell_reads for m in [tool_rx.search(e["note"])] if m)
    print(f"\nstore read via shell instead of the CLI: {len(shell_reads)} calls in {len({e['session'] for e in shell_reads})} sessions; "
          f"by tool {by_tool.most_common()}; `search` verb: {searches} commands")
    archived = sum(1 for e in hand if "/archive/" in e["note"])
    print(f"hand edits of task files: {len(hand)} in {len({e['session'] for e in hand})} sessions; by tool:",
          Counter(e["tool"] for e in hand).most_common(), f"; on archived files: {archived}")
    out["store_shell_reads"], out["searches"], out["hand_edits"] = len(shell_reads), searches, len(hand)

    # ---- cross-repo hops ---------------------------------------------------------------------
    hops = [e for e in calls if e["hop"]]
    targets = Counter(m.group(1) for e in hops for m in [HOP.search(e["note"])] if m)
    hop_verbs = Counter(v for e in hops for v in e["verbs"])
    print(f"\ncross-repo hops (cd ../x && meshwork …): {len(hops)} calls in {len({e['session'] for e in hops})} sessions; "
          f"targets {targets.most_common(8)}; verbs {hop_verbs.most_common(8)}")
    out["hops"] = {"calls": len(hops), "targets": dict(targets), "verbs": dict(hop_verbs)}

    # ---- output volume -----------------------------------------------------------------------
    big = Counter()
    for e in calls:
        if e.get("out_bytes", 0) > 12000:
            big[e["verbs"][0]] += 1
    print("results over 12 KB by verb:", big.most_common(10))
    out["big_outputs"] = dict(big)

    # ---- bookkeeping share & comment volume --------------------------------------------------
    shares = [r["mw_cmds"] / r["n_tool"] for r in used if r["n_tool"]]
    comments = [e for e in calls if "comment" in e["verbs"]]
    clen = [len(e["note"]) for e in comments]
    print(f"\nmeshwork share of all tool calls (sessions that used it): median {statistics.median(shares):.2f}, "
          f"p90 {sorted(shares)[int(.9 * len(shares))]:.2f}")
    print(f"comments: {len(comments)} calls; median command length {statistics.median(clen) if clen else 0:.0f} chars "
          f"(300 = truncated); @file/stdin bodies: {sum('@' in e['note'] or ' - ' in e['note'] for e in comments)}")
    out["mw_share_median"] = statistics.median(shares) if shares else None

    # ---- weekly trend ------------------------------------------------------------------------
    weeks = defaultdict(Counter)
    for r in rows:
        if not r["start"]:
            continue
        wk = datetime.fromisoformat(r["start"].replace("Z", "+00:00")).strftime("%G-W%V")
        w = weeks[wk]
        w["sessions"] += 1
        w["used"] += 1 if r["mw_cmds"] else 0
        for k in ("mw_cmds", "mw_err", "mw_guess", "mw_help", "store_hand_edit", "heated"):
            w[k] += r[k]
    print("\nweek\tsessions\tused\tcmds\terr\tguess\thelp\thand\theated")
    for wk in sorted(weeks):
        w = weeks[wk]
        print("\t".join(str(x) for x in [wk, w["sessions"], w["used"], w["mw_cmds"], w["mw_err"], w["mw_guess"],
                                         w["mw_help"], w["store_hand_edit"], w["heated"]]))
    out["weeks"] = {k: dict(v) for k, v in weeks.items()}

    denied = [e for e in calls if e.get("denied")]
    print(f"\ndenied meshwork calls: {len(denied)}")
    for e in denied[:12]:
        print(f"  [{e['repo']}/{e['session'][:8]} t{e['turn']}] {e['note'][:160]}")

    out["waits"] = wait_table(rows)
    if args.json:
        json.dump(out, open(args.json, "w"), indent=1)


if __name__ == "__main__":
    main()
