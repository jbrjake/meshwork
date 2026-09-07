#!/usr/bin/env python3
"""Rung-0 baseline for docs/PROPOSAL-prioritization.md — read-only.

Per-repo `meshwork q --json` over tasks/edges/log for every registered store (never a
`portfolio` verb: those autoprune sequence.md as a side effect of reading). Prints, with
denominators: placement today (seq vs none, category coverage), cycle/queue/service times,
the discrete daily close hazard with competing risks and equalized follow-up, live tasks past
the triage age, lane structure, and the Sidney-vs-inheritance experiment on each ready set.

    python3 scripts/mine_rank.py            # human tables
    python3 scripts/mine_rank.py --json     # one JSON document
    python3 scripts/mine_rank.py --check    # exit 0 iff every denominator is non-zero

Env: MESHWORK_BIN (default target/debug/meshwork), MESHWORK_CODE_ROOT (default ~/Documents/code),
MESHWORK_NOW (ISO minute stamp; default: now UTC), MESHWORK_REPOS (comma list; default: the
registry's names read from the portfolio repo's repos.toml when present, else a built-in list).
"""
import datetime as dt
import json
import os
import statistics
import subprocess
import sys
from collections import Counter, defaultdict, deque

ROOT = os.path.expanduser(os.environ.get("MESHWORK_CODE_ROOT", "~/Documents/code"))
BIN = os.environ.get("MESHWORK_BIN") or os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "target", "debug", "meshwork")
LIVE = ("open", "doing", "blocked")
TRIAGE_DAYS = 14
FALLBACK_REPOS = ["meshwork", "sazed", "leras", "marasi", "tensoon", "oreseur", "wyndam",
                  "marasi-applied-r-and-d", "portfolio"]


def now_utc():
    s = os.environ.get("MESHWORK_NOW")
    if s:
        return dt.datetime.strptime(s[:16], "%Y-%m-%dT%H:%M").replace(tzinfo=dt.timezone.utc)
    return dt.datetime.now(dt.timezone.utc).replace(second=0, microsecond=0)


def repos():
    env = os.environ.get("MESHWORK_REPOS")
    if env:
        return [r.strip() for r in env.split(",") if r.strip()]
    toml_path = os.path.join(ROOT, "portfolio", "repos.toml")
    names = []
    if os.path.exists(toml_path):
        with open(toml_path, encoding="utf-8") as f:
            for line in f:
                line = line.strip()
                if not line.startswith("name") or "=" not in line:
                    continue
                rhs = line.split("=", 1)[1]
                if rhs.count('"') >= 2:          # name = "x"   # trailing comment tolerated
                    v = rhs.split('"')[1]
                    if v:
                        names.append(v)
    return names or FALLBACK_REPOS


def q(repo, sql):
    cwd = os.path.join(ROOT, repo)
    if not os.path.isdir(cwd):
        return None
    r = subprocess.run([BIN, "--json", "q", sql], cwd=cwd, capture_output=True, text=True)
    if r.returncode != 0:
        sys.stderr.write(f"{repo}: {r.stderr.strip()[:160]}\n")
        return None
    d = json.loads(r.stdout)["data"]
    return [dict(zip(d["columns"], row)) for row in d["rows"]]


def stamp(s):
    if not s:
        return None
    s = s.strip()
    try:
        if len(s) == 10:
            return dt.datetime.strptime(s, "%Y-%m-%d").replace(tzinfo=dt.timezone.utc)
        return dt.datetime.strptime(s[:16], "%Y-%m-%dT%H:%M").replace(tzinfo=dt.timezone.utc)
    except ValueError:
        return None


def hours(a, b):
    return (b - a).total_seconds() / 3600.0


def med(xs):
    return statistics.median(xs) if xs else None


def p90(xs):
    if not xs:
        return None
    xs = sorted(xs)
    return xs[min(len(xs) - 1, int(0.9 * len(xs)))]


def cat2(c):
    return "/".join(c.split("/")[:2]) if c else "(none)"


# --- graph helpers -------------------------------------------------------------------------

def components(nodes, pairs):
    adj = defaultdict(set)
    ns = set(nodes)
    for a, b in pairs:
        if a in ns and b in ns:
            adj[a].add(b)
            adj[b].add(a)
    seen, sizes = set(), []
    for g in nodes:
        if g in seen:
            continue
        stack, n = [g], 0
        seen.add(g)
        while stack:
            x = stack.pop()
            n += 1
            for y in adj[x]:
                if y not in seen:
                    seen.add(y)
                    stack.append(y)
        sizes.append(n)
    return sorted(sizes, reverse=True)


def max_closure(nodes, w, p, lam, needs):
    """Max-weight set closed under prerequisites, via Edmonds-Karp min-cut (Picard).
    needs: (dependent, prerequisite) pairs — meshwork's stored orientation IS the arc direction."""
    idx = {x: i for i, x in enumerate(nodes)}
    n = len(nodes) + 2
    s, t = n - 2, n - 1
    cap = {}
    for x in nodes:
        a = w[x] - lam * p[x]
        if a > 0:
            cap[(s, idx[x])] = a
        elif a < 0:
            cap[(idx[x], t)] = -a
    for d, pr in needs:
        cap[(idx[d], idx[pr])] = 1e18
    adj = defaultdict(set)
    for (u, v) in cap:
        adj[u].add(v)
        adj[v].add(u)
    flow = defaultdict(float)

    def resid(u, v):
        return cap.get((u, v), 0.0) - flow[(u, v)] + flow[(v, u)]

    while True:
        par = {s: None}
        dq = deque([s])
        while dq and t not in par:
            u = dq.popleft()
            for v in adj[u]:
                if v not in par and resid(u, v) > 1e-9:
                    par[v] = u
                    dq.append(v)
        if t not in par:
            break
        b, v = float("inf"), t
        while par[v] is not None:
            b = min(b, resid(par[v], v))
            v = par[v]
        v = t
        while par[v] is not None:
            flow[(par[v], v)] += b
            v = par[v]
    seen, dq = {s}, deque([s])
    while dq:
        u = dq.popleft()
        for v in adj[u]:
            if v not in seen and resid(u, v) > 1e-9:
                seen.add(v)
                dq.append(v)
    return {nodes[i] for i in seen if i < len(nodes)}


def sidney_blocks(nodes, w, p, needs):
    blocks, rem = [], list(nodes)
    while rem:
        rs = set(rem)
        e = [(d, pr) for d, pr in needs if d in rs and pr in rs]
        lam = sum(w[x] for x in rem) / sum(p[x] for x in rem)
        best = None
        for _ in range(64):
            cl = max_closure(rem, w, p, lam, e)
            if not cl:
                break
            l2 = sum(w[x] for x in cl) / sum(p[x] for x in cl)
            if best is not None and l2 <= lam + 1e-12:
                break
            best, lam = cl, l2
        if not best:
            best = set(rem)
        blocks.append(sorted(best))
        rem = [x for x in rem if x not in best]
    return blocks


# --- per-repo analysis ----------------------------------------------------------------------

def analyze(repo, now):
    tasks = q(repo, "SELECT gid, id, status, category, seq, created FROM tasks")
    if tasks is None:
        return None
    log = q(repo, "SELECT gid, date, to_status FROM log") or []
    edges = q(repo, "SELECT src_gid, dst_gid, kind FROM edges WHERE kind IN ('needs','parent')") or []
    by = {t["gid"]: t for t in tasks}
    first_doing, last_done, last_drop = {}, {}, {}
    for l in log:
        d = stamp(l["date"])
        if d is None:
            continue
        if l["to_status"] == "doing":
            first_doing[l["gid"]] = min(first_doing.get(l["gid"], d), d)
        elif l["to_status"] == "done":
            last_done[l["gid"]] = max(last_done.get(l["gid"], d), d)
        elif l["to_status"] == "dropped":
            last_drop[l["gid"]] = max(last_drop.get(l["gid"], d), d)
    live = [t for t in tasks if t["status"] in LIVE]
    r = {"tasks": len(tasks), "live": len(live), "done": sum(1 for t in tasks if t["status"] == "done"),
         "live_seq": sum(1 for t in live if t["seq"] is not None),
         "live_categorized": sum(1 for t in live if t["category"]),
         "categories": len({t["category"] for t in live if t["category"]})}
    cycle, queue, service, survival = [], [], [], []
    for t in tasks:
        c = stamp(t["created"])
        if c is None:
            continue
        if t["status"] == "done":
            dn = last_done.get(t["gid"])
            if dn is None or dn < c:
                continue
            cycle.append(hours(c, dn))
            survival.append((hours(c, dn), "done", hours(c, now)))
            s = first_doing.get(t["gid"])
            if s is not None and c <= s <= dn:
                queue.append(hours(c, s))
                service.append(hours(s, dn))
        elif t["status"] == "dropped":
            e = last_drop.get(t["gid"], now)
            survival.append((hours(c, e), "dropped", hours(c, now)))
        elif t["status"] in LIVE:
            survival.append((hours(c, now), "open", hours(c, now)))
    r.update({"closes_dated": len(cycle), "closes_with_start": len(service),
              "cycle_med_h": med(cycle), "cycle_p90_h": p90(cycle),
              "queue_med_h": med(queue), "service_med_h": med(service), "service_p90_h": p90(service)})
    old = [t for t in live if stamp(t["created"]) and hours(stamp(t["created"]), now) >= TRIAGE_DAYS * 24]
    r["past_triage"] = len(old)
    r["past_triage_seq"] = sum(1 for t in old if t["seq"] is not None)
    r["past_triage_categories"] = Counter(cat2(t["category"]) for t in old).most_common(5)
    live_g = {t["gid"] for t in live}
    needs = [(e["src_gid"], e["dst_gid"]) for e in edges if e["kind"] == "needs" and e["src_gid"] in live_g and e["dst_gid"] in live_g]
    both = [(e["src_gid"], e["dst_gid"]) for e in edges if e["src_gid"] in live_g and e["dst_gid"] in live_g]
    r["live_needs_edges"] = len(needs)
    r["lanes_needs_multi"] = [s for s in components(list(live_g), needs) if s > 1]
    r["lanes_needs_parent_multi"] = [s for s in components(list(live_g), both) if s > 1]
    rev = defaultdict(set)
    for d, pr in needs:
        rev[pr].add(d)

    def dependents(g):
        seen, st = set(), [g]
        while st:
            x = st.pop()
            for y in rev[x]:
                if y not in seen:
                    seen.add(y)
                    st.append(y)
        return seen
    r["unlock_gt0"] = sum(1 for g in live_g if dependents(g))
    needs_of = defaultdict(set)
    for e in edges:
        if e["kind"] == "needs":
            needs_of[e["src_gid"]].add(e["dst_gid"])
    ready = [t for t in live if t["status"] == "open" and all(by.get(d, {}).get("status") in ("done", "dropped") for d in needs_of[t["gid"]])]
    place = {t["gid"]: (t["seq"] if t["seq"] is not None else 999999) for t in live}
    w = {g: max(1.0, 1001 - min(place[g], 1000)) for g in live_g}
    p = {g: 1.0 for g in live_g}
    blocks = sidney_blocks(sorted(live_g), w, p, needs)
    blk = {g: i for i, b in enumerate(blocks) for g in b}
    key_today = lambda t: (place[t["gid"]], t["created"] or "", t["id"])
    today = sorted(ready, key=key_today)
    sid = sorted(ready, key=lambda t: (blk[t["gid"]],) + key_today(t))
    inherit = {t["gid"]: min([place[t["gid"]]] + [place[d] for d in dependents(t["gid"])]) for t in ready}
    r["ready"] = len(ready)
    r["sidney_blocks"] = len(blocks)
    r["sidney_positions_moved"] = sum(1 for i, t in enumerate(today) if sid[i] is not t)
    r["sidney_top8_changed"] = [t["id"] for t in today[:8]] != [t["id"] for t in sid[:8]]
    r["inheritance_findings"] = [(t["id"], place[t["gid"]], inherit[t["gid"]]) for t in ready if inherit[t["gid"]] < place[t["gid"]]]
    return r, survival


def hazard_table(survival):
    rows = []
    for a in list(range(0, 16)) + [16, 18, 20, 22, 24, 30]:
        risk = [s for s in survival if s[0] >= a * 24 and s[2] >= (a + 1) * 24]
        done = sum(1 for s in risk if s[1] == "done" and s[0] < (a + 1) * 24)
        drop = sum(1 for s in risk if s[1] == "dropped" and s[0] < (a + 1) * 24)
        if risk:
            rows.append({"age_d": a, "at_risk": len(risk), "done": done, "dropped": drop,
                         "h_done": round(done / len(risk), 3), "h_drop": round(drop / len(risk), 3)})
    return rows


def main():
    now = now_utc()
    out, pooled = {}, []
    for repo in repos():
        res = analyze(repo, now)
        if res is None:
            continue
        out[repo], surv = res
        pooled.extend(surv)
    doc = {"as_of": now.strftime("%Y-%m-%dT%H:%MZ"), "triage_days": TRIAGE_DAYS, "repos": out,
           "hazard": hazard_table(pooled)}
    if "--json" in sys.argv:
        print(json.dumps(doc, indent=1, default=str))
        return 0
    tot = lambda k: sum(v[k] for v in out.values())
    print(f"rank baseline as of {doc['as_of']} — {len(out)} stores, live {tot('live')}, seq'd {tot('live_seq')}, categorized {tot('live_categorized')}, past {TRIAGE_DAYS}d {tot('past_triage')} (seq'd {tot('past_triage_seq')}), closes dated {tot('closes_dated')} with start {tot('closes_with_start')}")
    for repo, r in out.items():
        sm = r["service_med_h"]
        print(f"\n== {repo}: live {r['live']} (seq'd {r['live_seq']}, categorized {r['live_categorized']}/{r['categories']} cats) done {r['done']}")
        print(f"   cycle med {r['cycle_med_h'] and round(r['cycle_med_h'], 1)} h p90 {r['cycle_p90_h'] and round(r['cycle_p90_h'], 1)} | queue med {r['queue_med_h'] and round(r['queue_med_h'], 1)} h | service med {sm and round(sm, 2)} h p90 {r['service_p90_h'] and round(r['service_p90_h'], 1)} (n {r['closes_with_start']}/{r['closes_dated']})")
        print(f"   past {TRIAGE_DAYS}d: {r['past_triage']} (seq'd {r['past_triage_seq']}) {r['past_triage_categories']}")
        print(f"   live needs {r['live_needs_edges']} | unlock>0 {r['unlock_gt0']} | lanes needs-only {r['lanes_needs_multi']} needs+parent {r['lanes_needs_parent_multi']}")
        print(f"   ready {r['ready']} | sidney blocks {r['sidney_blocks']} moved {r['sidney_positions_moved']} top8 changed {r['sidney_top8_changed']} | inheritance {r['inheritance_findings']}")
    print("\ndiscrete daily close hazard (pooled; at risk = open at age a and observable through a+1d):")
    print(" age  at-risk  done  drop  h_done  h_drop")
    for h in doc["hazard"]:
        print(f" {h['age_d']:3d}  {h['at_risk']:6d}  {h['done']:4d}  {h['dropped']:4d}  {h['h_done']:.3f}  {h['h_drop']:.3f}")
    if "--check" in sys.argv:
        bad = [k for k, v in out.items() if v["live"] == 0 or v["closes_dated"] == 0]
        if not out or bad:
            print(f"check: FAIL — empty denominators in {bad or 'all'}")
            return 1
        print("check: ok")
    return 0


if __name__ == "__main__":
    sys.exit(main())
