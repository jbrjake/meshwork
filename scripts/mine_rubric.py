#!/usr/bin/env python3
"""Rubric spike for docs/SPIKE-prioritization-rubric.md — read-only.

Per-repo `meshwork q --json` over the tables and the derived views for every registered store
(never a `portfolio` verb: those autoprune sequence.md as a side effect of reading). For every
candidate dimension it measures discriminating power on each store's READY set — the only set
a rank orders — then scores a declared-weight rubric, compares its order with today's, and
names every task that moved together with the term that moved it.

    python3 scripts/mine_rubric.py              # markdown tables
    python3 scripts/mine_rubric.py --json       # one JSON document
    python3 scripts/mine_rubric.py --falsify    # the disagreement list only
    python3 scripts/mine_rubric.py --check      # exit 0 iff every store loaded with a non-empty ready set

Env: MESHWORK_BIN, MESHWORK_CODE_ROOT, MESHWORK_REPOS, MESHWORK_NOW as mine_rank.py;
MESHWORK_COST_JSON = the output of `mine_cost.py --json` (per-task tokens; optional — the cost
dimension reports "no data" without it). Weights: WEIGHTS below — policy, declared, not fitted.
"""
import json
import math
import os
import re
import sys
from collections import Counter, defaultdict

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import mine_rank  # noqa: E402

TOP = 10
# Declared policy (one shared table for every store). Units are STEPS: one step is the store's
# median gap between neighbouring distinct `seq` values on its ready set (measured per store,
# never authored), so "−1 step" means "one hand-placed neighbour sooner" whether the store
# numbers by 1 (sazed), 5 (leras) or 10 (meshwork). Override on the command line for a
# sensitivity run: --weights unlock=-2,age=-1
WEIGHTS = {
    "unlock": -1.0,      # × log2(1 + live transitive dependents)
    "almost": -2.0,      # × the best of sibling / spawn-sibling / doc-peer done shares
    "docs": -0.5,        # × log2(1 + done tasks sharing a docs: link, anchor-level)
    "cost": 0.5,         # × log2(category median tokens / corpus median); cheaper sooner
    "age": 0.0,          # × age in weeks — triage, never a boost (C.4); 0 by policy, measured anyway
    "inherit": 1.0,      # 1 = place := min(place, inherit) when a live dependent is placed sooner; 0 = off
}
DEFAULT_PLACE = None     # an unplaced task's place: None = the store's median ready place; or a number


# --- inputs ---------------------------------------------------------------------------------

def load(repo):
    tasks = mine_rank.q(repo, "SELECT gid, id, title, status, category, seq, created, parent, path FROM tasks")
    if tasks is None:
        return None
    graph = mine_rank.q(repo, "SELECT gid, unlock, depth, place, inherit, lane_size, ready FROM graph") or []
    facts = mine_rank.q(repo, "SELECT gid, age_h, queue_h, live FROM facts") or []
    lineage = mine_rank.q(repo, "SELECT gid, spawned_total, spawned_done, descendants_total, descendants_done FROM lineage") or []
    edges = mine_rank.q(repo, "SELECT src_gid, dst_gid, kind FROM edges WHERE kind IN ('parent','discovered-from')") or []
    by = {t["gid"]: dict(t) for t in tasks}
    for rows in (graph, facts, lineage):
        for r in rows:
            if r["gid"] in by:
                by[r["gid"]].update(r)
    root = os.path.join(mine_rank.ROOT, repo)
    for t in by.values():
        t["docs"] = docs_of(root, t.get("path"))
    return by, edges


def docs_of(root, path):
    """The `docs:` list of one task file — inline or block form, frontmatter only."""
    if not path:
        return []
    full = path if os.path.isabs(path) else os.path.join(root, path)
    try:
        with open(full, encoding="utf-8") as f:
            text = f.read()
    except OSError:
        return []
    if not text.startswith("---"):
        return []
    end = text.find("\n---", 3)
    fm = text[3:end] if end > 0 else text[3:]
    out, block = [], False
    for line in fm.splitlines():
        if block:
            m = re.match(r"\s+-\s+(.+)$", line)
            if m:
                out.append(m.group(1).strip().strip('"'))
                continue
            block = False
        m = re.match(r"docs:\s*(.*)$", line)
        if m:
            rest = m.group(1).strip()
            if rest.startswith("["):
                out.extend(x.strip().strip('"') for x in rest.strip("[]").split(",") if x.strip())
            elif rest:
                out.append(rest.strip('"'))
            else:
                block = True
    return out


def cost_table():
    path = os.environ.get("MESHWORK_COST_JSON")
    if not path or not os.path.exists(path):
        return None
    with open(path, encoding="utf-8") as f:
        d = json.load(f)
    return d.get("tasks", {})


# --- dimensions -------------------------------------------------------------------------------

def truthy(v):
    return v is True or v == "true"


def num(v):
    try:
        return None if v is None else float(v)
    except (TypeError, ValueError):
        return None


def dimensions(by, edges, cost):
    """Every candidate dimension for every task; None = the dimension has nothing to say."""
    parent_of = {e["src_gid"]: e["dst_gid"] for e in edges if e["kind"] == "parent"}
    origin_of = {e["src_gid"]: e["dst_gid"] for e in edges if e["kind"] == "discovered-from"}
    kids, spawn = defaultdict(list), defaultdict(list)
    for c, p in parent_of.items():
        kids[p].append(c)
    for c, o in origin_of.items():
        spawn[o].append(c)
    by_link, by_path = defaultdict(set), defaultdict(set)
    for g, t in by.items():
        for d in t["docs"]:
            by_link[d].add(g)
            by_path[d.split("#")[0]].add(g)
    cats = {c["category"]: c for c in (cost or {}).get("cats", []) if c["n"] >= 7}
    corpus_med = cost.get("fresh_med") if cost else None

    def done_share(members, me):
        others = [m for m in members if m != me and m in by]
        if not others:
            return None
        return sum(1 for m in others if by[m]["status"] == "done") / len(others)

    dims = {}
    for g, t in by.items():
        d = {}
        d["unlock"] = num(t.get("unlock"))
        d["depth"] = num(t.get("depth"))
        age = num(t.get("age_h"))
        d["age_d"] = age / 24 if age is not None else None
        d["queue_h"] = num(t.get("queue_h"))
        d["seq"] = num(t.get("seq"))
        place = num(t.get("place")) or (d["seq"] if d["seq"] is not None else 999999)
        inherit = num(t.get("inherit"))
        d["inherit_gain"] = (place - inherit) if inherit is not None and inherit < place else 0.0
        d["sib_done"] = done_share(kids.get(parent_of.get(g), []), g) if g in parent_of else None
        d["spawn_done"] = done_share(spawn.get(origin_of.get(g), []), g) if g in origin_of else None
        peers, peers_path = set(), set()
        for link in t["docs"]:
            peers |= by_link[link]
            peers_path |= by_path[link.split("#")[0]]
        peers.discard(g)
        peers_path.discard(g)
        d["doc_peers_done"] = sum(1 for p in peers if by[p]["status"] == "done") if t["docs"] else None
        d["doc_done_share"] = (d["doc_peers_done"] / len(peers)) if peers else None
        d["docpath_peers_done"] = sum(1 for p in peers_path if by[p]["status"] == "done") if t["docs"] else None
        c2 = mine_rank.cat2(t.get("category"))
        row = cats.get(c2) or cats.get(t.get("category") or "")
        d["cost_k"] = (row["med"] / 1000) if row and row.get("med") else None
        d["cost_ratio"] = (row["med"] / corpus_med) if row and row.get("med") and corpus_med else None
        dims[g] = d
    return dims


# The null every dimension has to beat, stated before the numbers: "at its default on nearly
# every ready task" — a dimension that is null (None) or at NULL_VALUE says nothing about a task.
NULL_VALUE = {"unlock": 0, "depth": 0, "age_d": None, "queue_h": None, "seq": None, "inherit_gain": 0,
              "sib_done": None, "spawn_done": None, "doc_peers_done": 0, "doc_done_share": None,
              "docpath_peers_done": 0, "cost_k": None, "cost_ratio": None}
DIM_ORDER = ["seq", "unlock", "depth", "inherit_gain", "age_d", "queue_h", "sib_done", "spawn_done",
             "doc_peers_done", "doc_done_share", "docpath_peers_done", "cost_k"]
DIM_VIEW = {"seq": "tasks.seq", "unlock": "graph.unlock", "depth": "graph.depth", "inherit_gain": "graph.inherit − place",
            "age_d": "facts.age_h", "queue_h": "facts.queue_h", "sib_done": "tasks.parent + siblings' status",
            "spawn_done": "edges discovered-from + siblings' status", "doc_peers_done": "docs: links (files) + peers' status",
            "doc_done_share": "docs: links (files) + peers' status", "docpath_peers_done": "docs: paths (files) + peers' status",
            "cost_k": "cost-baseline: category median tokens"}


def power(dims, ready, today):
    """Coverage, activity, distinct values, and the share of pairs a dimension strictly orders —
    on the whole ready set and on today's top-TOP."""
    out = {}
    top = today[:TOP]
    for k in DIM_ORDER:
        vals = {g: dims[g][k] for g in ready}
        cov = [g for g in ready if vals[g] is not None]
        act = [g for g in cov if vals[g] != NULL_VALUE[k]]
        distinct = len({vals[g] for g in cov})

        def pairs(gs):
            n = len(gs)
            if n < 2:
                return None
            tot = n * (n - 1) // 2
            dec = sum(1 for i in range(n) for j in range(i + 1, n)
                      if vals[gs[i]] is not None and vals[gs[j]] is not None and vals[gs[i]] != vals[gs[j]])
            return dec, tot
        out[k] = {"n": len(ready), "cov": len(cov), "act": len(act), "distinct": distinct,
                  "pairs": pairs(list(ready)), "pairs_top": pairs(top)}
    return out


# --- the rubric -------------------------------------------------------------------------------

def scale(dims, ready):
    """The store's step (median gap between neighbouring distinct ready places) and the place an
    unplaced task takes (the median ready place) — both measured, neither authored."""
    places = sorted({dims[g]["seq"] for g in ready if dims[g]["seq"] is not None})
    gaps = [b - a for a, b in zip(places, places[1:]) if b > a]
    step = mine_rank.med(gaps) or 1.0
    default = DEFAULT_PLACE if DEFAULT_PLACE is not None else (mine_rank.med(places) or 500)
    return step, default


def terms(g, dims, w=WEIGHTS):
    """Every weighted term's contribution, in the rubric's unit (steps or positions); `seq` is
    the place as written and rides along for rendering. `inherit` is handled by the caller —
    it replaces the base, it does not add to it."""
    d = dims[g]
    t = {"seq": d["seq"]}
    if d["unlock"]:
        t["unlock"] = w["unlock"] * math.log2(1 + d["unlock"])
    almost = max([x for x in (d["sib_done"], d["spawn_done"], d["doc_done_share"]) if x is not None] or [0])
    if almost:
        t["almost"] = w["almost"] * almost
    if d["doc_peers_done"]:
        t["docs"] = w["docs"] * math.log2(1 + d["doc_peers_done"])
    if d["cost_ratio"] and w["cost"]:
        t["cost"] = w["cost"] * math.log2(d["cost_ratio"])
    if w["age"] and d["age_d"] is not None:
        t["age"] = w["age"] * d["age_d"] / 7
    return t


def render_terms(t, inherit_note=None):
    parts = ["seq —" if t["seq"] is None else f"seq {t['seq']:.0f}"]
    if inherit_note:
        parts.append(inherit_note)
    parts += [f"{k} {v:+.1f}" for k, v in t.items() if k != "seq"]
    return " · ".join(parts)


def falsify(by, dims, ready, w=WEIGHTS, space="rank"):
    """Today's order against the rubric's. `space` = "rank": the base is the task's position in
    today's order and a term moves it that many POSITIONS — invariant to how a store numbers
    `seq`. "points": the base is the place and a term moves it that many STEPS of the store's
    measured gap. An unplaced task starts at the middle of the placed ones either way; a task
    whose live dependent is placed sooner starts where that dependent starts (inherit)."""
    key_today = lambda g: (dims[g]["seq"] if dims[g]["seq"] is not None else 999999, by[g].get("created") or "", g)
    today = sorted(ready, key=key_today)
    pos_t = {g: i for i, g in enumerate(today)}
    step, default = scale(dims, ready)
    placed = [g for g in today if dims[g]["seq"] is not None]
    unit = 1.0 if space == "rank" else step

    def base_of(g):
        d = dims[g]
        if space == "rank":
            own = pos_t[g] + 1 if d["seq"] is not None else (len(placed) + 1) // 2 + 1
            if w["inherit"] and d["inherit_gain"]:
                inh = d["seq"] - d["inherit_gain"]
                own = min(own, sum(1 for p in placed if dims[p]["seq"] < inh) + 1)
            return own
        own = d["seq"] if d["seq"] is not None else default
        if w["inherit"] and d["inherit_gain"]:
            own = min(own, d["seq"] - d["inherit_gain"])
        return own

    base = {g: base_of(g) for g in ready}
    tm = {g: terms(g, dims, w) for g in ready}
    sc = {g: base[g] + unit * sum(v for k, v in tm[g].items() if k != "seq") for g in ready}
    rubric = sorted(ready, key=lambda g: (sc[g], by[g].get("created") or "", g))
    pos_r = {g: i for i, g in enumerate(rubric)}
    moved = []
    for g in ready:
        if pos_t[g] == pos_r[g] or (pos_t[g] >= TOP and pos_r[g] >= TOP):
            continue
        t = tm[g]
        d = dims[g]
        inherit_note = None
        if w["inherit"] and d["inherit_gain"]:
            inherit_note = f"inherit → {'#' if space == 'rank' else ''}{base[g]:g}"
        cause = max((k for k in t if k != "seq"), key=lambda k: abs(t[k]), default=None)
        own = abs(sum(v for k, v in t.items() if k != "seq"))
        if inherit_note and (cause is None or abs(base[g] - (pos_t[g] + 1 if space == "rank" else (d["seq"] or default))) > abs(t.get(cause, 0)) * unit):
            cause = "inherit"
        elif cause is None or (own < 0.5 and d["seq"] is not None):
            cause = "displaced"  # its own terms move it under half a unit; others moved past it
        elif d["seq"] is None:
            cause = "unplaced"
        moved.append({"id": by[g]["id"], "today": pos_t[g] + 1, "rubric": pos_r[g] + 1, "cause": cause,
                      "terms": render_terms(t, inherit_note), "title": by[g]["title"][:70]})
    moved.sort(key=lambda m: (m["rubric"], m["today"]))
    top_t, top_r = set(today[:TOP]), set(rubric[:TOP])
    return {"today": [by[g]["id"] for g in today[:TOP]], "rubric": [by[g]["id"] for g in rubric[:TOP]],
            "step": step, "default_place": default, "space": space,
            "moved_any": sum(1 for g in ready if pos_t[g] != pos_r[g]),
            "moved_far": sum(1 for g in ready if abs(pos_t[g] - pos_r[g]) > 1),
            "top_members_changed": len(top_t ^ top_r) // 2, "moved_top": moved,
            "top8_changed": [by[g]["id"] for g in today[:8]] != [by[g]["id"] for g in rubric[:8]]}, today


# --- the effort question ----------------------------------------------------------------------

def effort(cost, all_dims, all_ready):
    """Does measured cost by category do an authored size's work? Coverage of the ready sets, and
    the share of per-task log-token variance a category explains (eta²), against the spread."""
    if not cost:
        return None
    cats = [c for c in cost.get("cats", []) if c["n"] >= 7]
    per = cost.get("per_task") or {}
    groups = defaultdict(list)
    for gid, t in per.items():
        if t.get("category") and t["fresh"] and any(c["category"] == t["category"] for c in cats):
            groups[t["category"]].append(math.log(t["fresh"]))
    allv = [v for vs in groups.values() for v in vs]
    eta2 = None
    if len(allv) > 2 and len(groups) > 1:
        gm = sum(allv) / len(allv)
        ss_b = sum(len(vs) * (sum(vs) / len(vs) - gm) ** 2 for vs in groups.values())
        ss_t = sum((v - gm) ** 2 for v in allv)
        eta2 = ss_b / ss_t if ss_t else None
    within = [(c["category"], c["n"], c["med"], c["p90"], (c["p90"] / c["med"]) if c["med"] else None) for c in cats]
    covered = sum(1 for g in all_ready if all_dims[g]["cost_k"] is not None)
    return {"cats_n7": len(cats), "spread": cost.get("spread"), "eta2": eta2, "per_task_n": len(allv),
            "within": within, "ready_covered": covered, "ready_n": len(all_ready)}


# --- output -----------------------------------------------------------------------------------

def pct(a, b):
    return f"{100 * a / b:.0f}%" if b else "—"


def fr(p):
    return "—" if p is None else f"{p[0]}/{p[1]} ({pct(p[0], p[1])})"


def hubs(all_by, n=12):
    """The docs: links most shared by DONE tasks across every store — the hub docs a
    doc-cohesion term boosts wholesale."""
    done_by_link, live_by_link = Counter(), Counter()
    for by in all_by:
        for t in by.values():
            for d in set(t["docs"]):
                if t["status"] == "done":
                    done_by_link[d] += 1
                elif t["status"] in mine_rank.LIVE:
                    live_by_link[d] += 1
    print("| docs: link | done tasks citing it | live tasks citing it |")
    print("|---|---:|---:|")
    for link, c in done_by_link.most_common(n):
        print(f"| `{link}` | {c} | {live_by_link[link]} |")


def weights_from_argv():
    w = dict(WEIGHTS)
    if "--weights" in sys.argv:
        spec = sys.argv[sys.argv.index("--weights") + 1]
        for kv in spec.split(","):
            k, v = kv.split("=")
            if k not in w:
                sys.exit(f"unknown weight {k}; known: {', '.join(w)}")
            w[k] = float(v)
    return w


def main():
    now = mine_rank.now_utc()
    cost = cost_table()
    w = weights_from_argv()
    space = sys.argv[sys.argv.index("--space") + 1] if "--space" in sys.argv else "rank"
    if space not in ("rank", "points"):
        sys.exit("--space takes rank or points")
    stores, pooled_dims, pooled_ready, pooled_power, all_by = {}, {}, [], defaultdict(lambda: Counter()), []
    for repo in mine_rank.repos():
        loaded = load(repo)
        if loaded is None:
            continue
        by, edges = loaded
        all_by.append(by)
        dims = dimensions(by, edges, cost)
        ready = sorted(g for g, t in by.items() if truthy(t.get("ready")))
        fal, today = falsify(by, dims, ready, w, space)
        pw = power(dims, ready, today)
        stores[repo] = {"live": sum(1 for t in by.values() if t["status"] in mine_rank.LIVE), "ready": len(ready),
                        "power": pw, "falsifier": fal,
                        "top": [{"id": by[g]["id"], "terms": render_terms(terms(g, dims, w))} for g in today[:TOP]]}
        pooled_dims.update(dims)
        pooled_ready.extend(ready)
        for k, v in pw.items():
            for f in ("n", "cov", "act"):
                pooled_power[k][f] += v[f]
            for f in ("pairs", "pairs_top"):
                if v[f]:
                    pooled_power[k][f + "_dec"] += v[f][0]
                    pooled_power[k][f + "_tot"] += v[f][1]
    eff = effort(cost, pooled_dims, pooled_ready)
    doc = {"as_of": now.strftime("%Y-%m-%dT%H:%MZ"), "weights": w, "default_place": DEFAULT_PLACE,
           "stores": stores, "pooled_power": {k: dict(v) for k, v in pooled_power.items()}, "effort": eff}
    if "--hubs" in sys.argv:
        hubs(all_by)
        return 0
    if "--check" in sys.argv:
        bad = [r for r, s in stores.items() if s["ready"] == 0]
        print(f"check: {'FAIL — empty ready sets in ' + str(bad) if bad or not stores else 'ok'}")
        return 1 if bad or not stores else 0
    if "--json" in sys.argv:
        print(json.dumps(doc, indent=1, default=str))
        return 0
    if "--falsify" not in sys.argv:
        print(f"rubric spike as of {doc['as_of']} — {len(stores)} stores, ready {len(pooled_ready)} of live {sum(s['live'] for s in stores.values())}\n")
        print("## Discriminating power, pooled over every store's ready set\n")
        print("| dimension | reads | covered | active (≠ null) | pairs ordered, whole ready set | pairs ordered, today's top 10 |")
        print("|---|---|---:|---:|---:|---:|")
        for k in DIM_ORDER:
            p = pooled_power[k]
            print(f"| `{k}` | {DIM_VIEW[k]} | {p['cov']}/{p['n']} ({pct(p['cov'], p['n'])}) | {p['act']}/{p['n']} ({pct(p['act'], p['n'])}) | "
                  f"{fr((p['pairs_dec'], p['pairs_tot']) if p['pairs_tot'] else None)} | {fr((p['pairs_top_dec'], p['pairs_top_tot']) if p['pairs_top_tot'] else None)} |")
        print("\n## Per store: active share of the ready set\n")
        print("| store | ready | " + " | ".join(f"`{k}`" for k in DIM_ORDER) + " |")
        print("|---|---:|" + "---:|" * len(DIM_ORDER))
        for repo, s in stores.items():
            print(f"| {repo} | {s['ready']} | " + " | ".join(pct(s['power'][k]['act'], s['ready']) for k in DIM_ORDER) + " |")
        if eff:
            print("\n## The effort question\n")
            print(f"- categories with n ≥ 7 costed tasks: {eff['cats_n7']}; spread of medians {eff['spread']:.1f}×; per-task rows {eff['per_task_n']}")
            print(f"- share of per-task log-token variance the category explains (η²): {eff['eta2']:.2f}" if eff["eta2"] is not None else "- η²: no per-task data")
            print(f"- ready tasks whose category has a costed row: {eff['ready_covered']}/{eff['ready_n']} ({pct(eff['ready_covered'], eff['ready_n'])})\n")
            print("| category | n | median | p90 | p90 ÷ median |")
            print("|---|---:|---:|---:|---:|")
            for c, n, m, p9, r in eff["within"]:
                print(f"| {c} | {n} | {m / 1000:.0f}k | {p9 / 1000:.0f}k | {r:.1f}× |" if r else f"| {c} | {n} | — | — | — |")
        print("\n## Falsifier — the declared rubric against today's order\n")
    unit = "positions" if space == "rank" else "steps of the store's measured seq gap"
    print(f"space: {space} (a term moves a task by {unit}); weights: {w}; unplaced → the middle of the placed tasks\n")
    print("| store | ready | step | positions moved | moved > 1 | top-10 members changed | top-8 order changed | rows below |")
    print("|---|---:|---:|---:|---:|---:|---|---:|")
    for repo, s in stores.items():
        f = s["falsifier"]
        print(f"| {repo} | {s['ready']} | {f['step']:g} | {f['moved_any']} | {f['moved_far']} | {f['top_members_changed']} | {'yes' if f['top8_changed'] else 'no'} | {len(f['moved_top'])} |")
    for repo, s in stores.items():
        f = s["falsifier"]
        if not f["moved_top"]:
            continue
        print(f"\n### {repo} — every task that moved within the top {TOP}\n")
        print("| id | today | rubric | term that moved it | terms | title |")
        print("|---|---:|---:|---|---|---|")
        for m in f["moved_top"]:
            print(f"| {m['id']} | {m['today']} | {m['rubric']} | {m['cause']} | {m['terms']} | {m['title']} |")
    return 0


if __name__ == "__main__":
    sys.exit(main())
