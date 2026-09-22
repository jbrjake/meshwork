#!/usr/bin/env python3
"""The Sidney-vs-inheritance experiment on each ready set — read-only.

Per-repo `meshwork q --json` over tasks/edges for every registered store (never a `portfolio`
verb: those autoprune sequence.md as a side effect of reading). For each repo: the ready set
ordered as today (place, created, id) against the same set ordered by Sidney blocks — maximum-
weight closures under live `needs`, the ordering docs/PROPOSAL-prioritization.md §7 rung 6
argues for — the positions that move, whether the top eight change, and the inheritance
findings (a ready task placed behind a live dependent, MW-R6c's opt-in). The baseline tables —
placement, cycle/queue/service times, the close hazard, triage age, lanes — are
`meshwork stats --json` and `meshwork portfolio stats --json`; this script keeps the one
experiment the views do not run.

    python3 scripts/mine_rank.py            # human lines
    python3 scripts/mine_rank.py --json     # one JSON document
    python3 scripts/mine_rank.py --check    # exit 0 iff every store has a non-empty ready set

Env: MESHWORK_BIN (default target/debug/meshwork), MESHWORK_CODE_ROOT (default ~/Documents/code),
MESHWORK_REPOS (comma list; default: the registry's names read from the portfolio repo's
repos.toml when present, else a built-in list).
"""
import json
import os
import subprocess
import sys
from collections import defaultdict, deque

ROOT = os.path.expanduser(os.environ.get("MESHWORK_CODE_ROOT", "~/Documents/code"))
BIN = os.environ.get("MESHWORK_BIN") or os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "target", "debug", "meshwork")
LIVE = ("open", "doing", "blocked")
FALLBACK_REPOS = ["meshwork", "sazed", "leras", "marasi", "tensoon", "oreseur", "wyndam",
                  "marasi-applied-r-and-d", "portfolio"]


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


# --- Sidney blocks ---------------------------------------------------------------------------

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


# --- per-repo experiment ---------------------------------------------------------------------

def analyze(repo):
    tasks = q(repo, "SELECT gid, id, status, seq, created FROM tasks")
    if tasks is None:
        return None
    edges = q(repo, "SELECT src_gid, dst_gid FROM edges WHERE kind = 'needs'") or []
    by = {t["gid"]: t for t in tasks}
    live = [t for t in tasks if t["status"] in LIVE]
    live_g = {t["gid"] for t in live}
    needs = [(e["src_gid"], e["dst_gid"]) for e in edges if e["src_gid"] in live_g and e["dst_gid"] in live_g]
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

    needs_of = defaultdict(set)
    for e in edges:
        needs_of[e["src_gid"]].add(e["dst_gid"])
    ready = [t for t in live if t["status"] == "open"
             and all(by.get(d, {}).get("status") in ("done", "dropped") for d in needs_of[t["gid"]])]
    place = {t["gid"]: (t["seq"] if t["seq"] is not None else 999999) for t in live}
    w = {g: max(1.0, 1001 - min(place[g], 1000)) for g in live_g}
    p = {g: 1.0 for g in live_g}
    blocks = sidney_blocks(sorted(live_g), w, p, needs)
    blk = {g: i for i, b in enumerate(blocks) for g in b}
    key_today = lambda t: (place[t["gid"]], t["created"] or "", t["id"])
    today = sorted(ready, key=key_today)
    sid = sorted(ready, key=lambda t: (blk[t["gid"]],) + key_today(t))
    inherit = {t["gid"]: min([place[t["gid"]]] + [place[d] for d in dependents(t["gid"])]) for t in ready}
    return {"live": len(live), "live_needs_edges": len(needs), "ready": len(ready),
            "sidney_blocks": len(blocks),
            "sidney_positions_moved": sum(1 for i, t in enumerate(today) if sid[i] is not t),
            "sidney_top8_changed": [t["id"] for t in today[:8]] != [t["id"] for t in sid[:8]],
            "inheritance_findings": [(t["id"], place[t["gid"]], inherit[t["gid"]])
                                     for t in ready if inherit[t["gid"]] < place[t["gid"]]]}


def main():
    out = {}
    for repo in repos():
        res = analyze(repo)
        if res is not None:
            out[repo] = res
    doc = {"repos": out}
    if "--json" in sys.argv:
        print(json.dumps(doc, indent=1, default=str))
        return 0
    print(f"sidney experiment — {len(out)} stores; the baseline tables are `meshwork stats --json`")
    for repo, r in out.items():
        print(f"== {repo}: live {r['live']} | live needs {r['live_needs_edges']} | ready {r['ready']} "
              f"| sidney blocks {r['sidney_blocks']} moved {r['sidney_positions_moved']} "
              f"top8 changed {r['sidney_top8_changed']} | inheritance {r['inheritance_findings']}")
    if "--check" in sys.argv:
        bad = [k for k, v in out.items() if v["ready"] == 0]
        if not out or bad:
            print(f"check: FAIL — empty ready sets in {bad or 'all'}")
            return 1
        print("check: ok")
    return 0


if __name__ == "__main__":
    sys.exit(main())
