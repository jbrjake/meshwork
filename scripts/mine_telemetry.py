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


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("scores")
    ap.add_argument("events")
    ap.add_argument("--json")
    args = ap.parse_args()
    rows = json.load(open(args.scores))
    by_sid = {r["session"]: r for r in rows}
    events = [json.loads(l) for l in open(args.events)]
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
    for e in events:
        if e["help"]:
            for v in e["verbs"]:
                helps[v] += 1
    print("\nverbs:", verbs.most_common(30))
    print("guessed verbs:", guessed.most_common(25))
    print("--help probes by verb:", helps.most_common(20), " total", sum(helps.values()))
    out["verbs"], out["guessed"], out["help"] = dict(verbs), dict(guessed), dict(helps)

    # ---- errors ------------------------------------------------------------------------------
    kinds, lines = Counter(), Counter()
    for e in events:
        if e.get("err"):
            kinds[e["err"]] += 1
            lines[(e["err"], norm(e.get("err_line", "")))] += 1
    total_calls = sum(1 for e in events if not e["verbs"][0].startswith(("hand-edit", "store-shell")))
    print(f"\nerrors: {sum(kinds.values())} of {total_calls} meshwork calls; kinds:", kinds.most_common())
    print("top normalized error lines:")
    for (k, l), n in lines.most_common(28):
        print(f"  {n:4d}  [{k}] {l}")
    out["err_kinds"] = dict(kinds)
    out["err_lines"] = [{"kind": k, "line": l, "n": n} for (k, l), n in lines.most_common(40)]

    # ---- close refusals → what happened next -------------------------------------------------
    res = Counter()
    sub = Counter()
    waives = sum(1 for e in events if e["waive"])
    set_verify_total = sum(1 for e in events if e["set_verify"])

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
            if e.get("err") != "close-refused" or not e["id"]:
                continue
            tid = e["id"]
            res["refusals"] += 1
            rk = refusal_kind(e.get("err_line", ""))
            sub[rk] += 1
            modified = None
            outcome = "never-closed"
            for f in evs[i + 1:]:
                if tid not in f.get("ids", []) and f["id"] != tid:
                    continue
                if f["set_verify"] or f["verbs"][0].startswith("hand-edit"):
                    modified = modified or ("set-verify" if f["set_verify"] else "hand-edit")
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
        first_start = next((e["id"] for e in evs if "start" in e["verbs"] and e["id"]), None)
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
            touched = any((e["id"] and e["id"][:2] != own) or e["hop"] or "answers" in e["cmd"] for e in evs)
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
    shell_reads = [e for e in events if e["verbs"][0] == "store-shell-read"]
    hand = [e for e in events if e["verbs"][0].startswith("hand-edit")]
    searches = sum(1 for e in events if "search" in e["verbs"])
    tool_rx = re.compile(r"\b(grep|rg|find|ls|cat|head|sed|awk)\b[^\n|]*docs/meshwork")
    by_tool = Counter(m.group(1) for e in shell_reads for m in [tool_rx.search(e["cmd"])] if m)
    print(f"\nstore read via shell instead of the CLI: {len(shell_reads)} calls in {len({e['session'] for e in shell_reads})} sessions; "
          f"by tool {by_tool.most_common()}; `search` verb: {searches} commands")
    archived = sum(1 for e in hand if "/archive/" in e["cmd"])
    print(f"hand edits of task files: {len(hand)} in {len({e['session'] for e in hand})} sessions; by tool:",
          Counter(e["verbs"][0] for e in hand).most_common(), f"; on archived files: {archived}")
    out["store_shell_reads"], out["searches"], out["hand_edits"] = len(shell_reads), searches, len(hand)

    # ---- cross-repo hops ---------------------------------------------------------------------
    hops = [e for e in events if e["hop"]]
    targets = Counter(m.group(1) for e in hops for m in [HOP.search(e["cmd"])] if m)
    hop_verbs = Counter(v for e in hops for v in e["verbs"])
    print(f"\ncross-repo hops (cd ../x && meshwork …): {len(hops)} calls in {len({e['session'] for e in hops})} sessions; "
          f"targets {targets.most_common(8)}; verbs {hop_verbs.most_common(8)}")
    out["hops"] = {"calls": len(hops), "targets": dict(targets), "verbs": dict(hop_verbs)}

    # ---- output volume -----------------------------------------------------------------------
    big = Counter()
    for e in events:
        if e.get("out_bytes", 0) > 12000:
            big[e["verbs"][0]] += 1
    print("results over 12 KB by verb:", big.most_common(10))
    out["big_outputs"] = dict(big)

    # ---- bookkeeping share & comment volume --------------------------------------------------
    shares = [r["mw_cmds"] / r["n_tool"] for r in used if r["n_tool"]]
    comments = [e for e in events if "comment" in e["verbs"]]
    clen = [len(e["cmd"]) for e in comments]
    print(f"\nmeshwork share of all tool calls (sessions that used it): median {statistics.median(shares):.2f}, "
          f"p90 {sorted(shares)[int(.9 * len(shares))]:.2f}")
    print(f"comments: {len(comments)} calls; median command length {statistics.median(clen) if clen else 0:.0f} chars "
          f"(300 = truncated); @file/stdin bodies: {sum('@' in e['cmd'] or ' - ' in e['cmd'] for e in comments)}")
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

    denied = [e for e in events if e.get("denied")]
    print(f"\ndenied meshwork calls: {len(denied)}")
    for e in denied[:12]:
        print(f"  [{e['repo']}/{e['session'][:8]} t{e['turn']}] {e['cmd'][:160]}")
    if args.json:
        json.dump(out, open(args.json, "w"), indent=1)


if __name__ == "__main__":
    main()
