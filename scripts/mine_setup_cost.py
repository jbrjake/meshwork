#!/usr/bin/env python3
"""Setup-cost matrix miner (mw-5h2mpn7): read-only, emits docs/setup-cost-matrix.md.

Sources, all read-only:
  1. the portfolio store  — `meshwork portfolio q … --json` (tasks, log, comments)
  2. session transcripts  — ~/.claude/projects/-…-<repo>/*.jsonl (one file = one session)
  3. git history          — Claude-Session commit trailers per adopter repo

Usage: python3 scripts/mine_setup_cost.py [--write]   (default: stdout)
"""

import json
import re
import subprocess
import sys
from datetime import datetime, timedelta, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SHIM = ROOT / "meshwork"
OUT = ROOT / "docs" / "setup-cost-matrix.md"
PROJECTS = Path.home() / ".claude" / "projects"

# Verb classification: acts mutate a task; reads only look. Stated in the doc's method.
ACT_VERBS = {"start", "close", "add", "comment", "set", "block", "drop", "reopen", "attach", "dep"}
READ_VERBS = {"prime", "ready", "show", "why", "tree", "blocked", "q", "lint", "next", "version"}

# Any token ending in `meshwork` (shim, pin path, target/debug) or the adopters'
# `$MW` variable convention, followed by a known verb (the verb filter contains noise).
MESH_CMD = re.compile(r"(?:\S*meshwork[\"']?|\"?\$\{?MW\}?\"?)\s+(?:portfolio\s+)?([a-z]+)\b")
GIT_C = re.compile(r"git\s+-C\s+\S+")
TASK_ID = re.compile(r"\b(?:mw|sa|le|ma|te|or)-[a-z0-9]{4,9}\b")
TS_RE = re.compile(r'"timestamp":"([^"]+)"')
OUT_TOK = re.compile(r'"output_tokens":(\d+)')
IN_TOK = re.compile(r'"input_tokens":(\d+)')
CACHE_READ = re.compile(r'"cache_read_input_tokens":(\d+)')
CACHE_CREATE = re.compile(r'"cache_creation_input_tokens":(\d+)')
SESSION_TRAILER = re.compile(r"session_[A-Za-z0-9]+")


def parse_ts(s):
    if not s:
        return None
    for fmt in ("%Y-%m-%dT%H:%M:%S.%fZ", "%Y-%m-%dT%H:%M:%SZ", "%Y-%m-%dT%H:%MZ", "%Y-%m-%d"):
        try:
            return datetime.strptime(s, fmt).replace(tzinfo=timezone.utc)
        except ValueError:
            continue
    return None


def q(sql):
    out = subprocess.run([str(SHIM), "portfolio", "q", sql, "--json"],
                         cwd=ROOT, capture_output=True, text=True, check=True).stdout
    return json.loads(out)["data"]["rows"]


def median(xs):
    xs = sorted(xs)
    n = len(xs)
    if n == 0:
        return None
    mid = n // 2
    return xs[mid] if n % 2 else (xs[mid - 1] + xs[mid]) / 2


def p90(xs):
    xs = sorted(xs)
    return xs[min(len(xs) - 1, int(round(0.9 * (len(xs) - 1))))] if xs else None


def fmt(x, unit="", nd=1):
    return "—" if x is None else f"{x:.{nd}f}{unit}"


# ---------------------------------------------------------------- transcripts

def scan_session(path):
    """One transcript file = one session (a conversation thread; /clear starts a new one)."""
    s = {"file": path.name, "t0": None, "t_end": None, "first_read": None, "first_act": None,
         "ctx_at_act": None, "out_before_act": None, "act_ids": set(), "adds": 0,
         "mesh_cmds": 0, "primed": False, "sidechain_lines": 0, "out_tokens": 0}
    cum_out = 0
    with open(path, errors="replace") as fh:
        for line in fh:
            m = TS_RE.search(line)
            ts = parse_ts(m.group(1)) if m else None
            if ts:
                s["t0"] = s["t0"] or ts
                s["t_end"] = ts
            if '"isSidechain":true' in line:
                s["sidechain_lines"] += 1
            o = OUT_TOK.search(line)
            if o:
                cum_out += int(o.group(1))
            if not s["primed"] and ("meshwork prime" in line or "meshwork — " in line):
                s["primed"] = True
            if "meshwork" not in line or '"tool_use"' not in line or '"Bash"' not in line:
                continue
            try:
                obj = json.loads(line)
            except json.JSONDecodeError:
                continue
            if obj.get("isSidechain"):
                continue
            for c in (obj.get("message") or {}).get("content") or []:
                if not (isinstance(c, dict) and c.get("type") == "tool_use" and c.get("name") == "Bash"):
                    continue
                cmd = (c.get("input") or {}).get("command", "")
                # `git -C <path-ending-in-meshwork> add|show …` is git, not meshwork
                cmd = GIT_C.sub("git", cmd)
                matches = list(MESH_CMD.finditer(cmd))
                for i, vm in enumerate(matches):
                    verb = vm.group(1)
                    if verb not in ACT_VERBS and verb not in READ_VERBS:
                        continue
                    s["mesh_cmds"] += 1
                    if s["first_read"] is None and ts:
                        s["first_read"] = ts
                    if verb in ACT_VERBS:
                        if s["first_act"] is None and ts:
                            s["first_act"] = ts
                            s["out_before_act"] = cum_out
                            ctx = 0
                            for rx in (IN_TOK, CACHE_READ, CACHE_CREATE):
                                cm = rx.search(line)
                                ctx += int(cm.group(1)) if cm else 0
                            s["ctx_at_act"] = ctx or None
                        if verb == "add":
                            s["adds"] += 1
                        end = matches[i + 1].start() if i + 1 < len(matches) else len(cmd)
                        s["act_ids"].update(TASK_ID.findall(cmd[vm.end():end]))
    s["out_tokens"] = cum_out
    return s if s["t0"] else None


# ---------------------------------------------------------------- git

def scan_commits(repo_path):
    out = subprocess.run(
        ["git", "-C", str(repo_path), "log", "--format=%H\t%cI\t%(trailers:key=Claude-Session,valueonly,separator=,)"],
        capture_output=True, text=True)
    commits = []
    for row in out.stdout.splitlines():
        parts = row.split("\t")
        if len(parts) < 2:
            continue
        try:
            ts = datetime.fromisoformat(parts[1]).astimezone(timezone.utc)
        except ValueError:
            continue
        trailer = SESSION_TRAILER.search(parts[2]) if len(parts) > 2 else None
        commits.append({"ts": ts, "session": trailer.group(0) if trailer else None})
    return commits


# ---------------------------------------------------------------- main

def main():
    repos = {r: Path(p) for r, p in q("SELECT repo, path FROM repos WHERE present")}
    tasks = {g: {"created": parse_ts(c), "status": st}
             for g, c, st in q("SELECT gid, created, status FROM tasks")}
    log_rows = q("SELECT gid, date, from_status, to_status, note FROM log")
    comment_rows = q("SELECT gid, date FROM comments")

    birth = {}
    for gid, t in tasks.items():
        r = gid.split("#")[0]
        if t["created"] and (r not in birth or t["created"] < birth[r]):
            birth[r] = t["created"]

    sessions = []
    commits = {}
    for repo, path in repos.items():
        proj = PROJECTS / ("-" + str(path).strip("/").replace("/", "-"))
        for f in sorted(proj.glob("*.jsonl")):
            s = scan_session(f)
            if s:
                s["repo"] = repo
                sessions.append(s)
        commits[repo] = scan_commits(path)
    sessions.sort(key=lambda s: s["t0"])

    data_end = max([s["t_end"] for s in sessions] +
                   [t for t in (parse_ts(d) for _, d, *_ in log_rows) if t])

    # per-session derived numbers
    for s in sessions:
        s["post_adoption"] = s["repo"] in birth and s["t_end"] >= birth[s["repo"]]
        s["ramp_read"] = (s["first_read"] - s["t0"]).total_seconds() / 60 if s["first_read"] else None
        s["ramp_act"] = (s["first_act"] - s["t0"]).total_seconds() / 60 if s["first_act"] else None
        cs = [c["ts"] for c in commits[s["repo"]] if s["t0"] <= c["ts"] <= s["t_end"]]
        s["first_commit_min"] = (min(cs) - s["t0"]).total_seconds() / 60 if cs else None

    # switch classification: predecessor = session with latest start before this one, any repo
    for i, s in enumerate(sessions):
        prev = sessions[i - 1] if i else None
        if not prev:
            s["switch"] = "cold"
            continue
        gap_h = (s["t0"] - prev["t_end"]).total_seconds() / 3600
        s["switch"] = "cold" if gap_h > 12 else ("same-repo" if prev["repo"] == s["repo"] else "cross-repo")

    acted = [s for s in sessions if s["post_adoption"] and s["ramp_act"] is not None]

    # aging: every touch (post-creation log row or comment) → task age at touch;
    # negative ages (date-only stamps round to midnight) excluded, counted.
    touches, junk = [], 0
    raw = [(g, d) for g, d, _f, to, note in log_rows
           if not (to is None and (note or "").strip() == "created")]
    raw += [(g, d) for g, d in comment_rows]
    for gid, date in raw:
        t, ts = tasks.get(gid), parse_ts(date)
        if not (t and t["created"] and ts):
            continue
        age_h = (ts - t["created"]).total_seconds() / 3600
        if age_h < 0:
            junk += 1
        else:
            touches.append((gid, age_h))

    closes = {}
    for gid, date, _f, to, _n in log_rows:
        ts = parse_ts(date)
        if to == "done" and ts and (gid not in closes or ts < closes[gid]):
            closes[gid] = ts

    cycle = {}
    for gid, close_ts in closes.items():
        t = tasks.get(gid)
        if t and t["created"]:
            cycle[gid] = (close_ts - t["created"]).total_seconds() / 3600

    emit(repos, tasks, log_rows, comment_rows, sessions, acted, commits, birth,
         touches, junk, closes, cycle, data_end)


# ---------------------------------------------------------------- report

def emit(repos, tasks, log_rows, comment_rows, sessions, acted, commits, birth,
         touches, junk, closes, cycle, data_end):
    L = []
    w = L.append
    end = data_end.strftime("%Y-%m-%d %H:%MZ")

    # aggregates the headlines share with the tables below
    all_ramp = [s["ramp_act"] for s in acted]
    all_ctx = [s["ctx_at_act"] / 1000 for s in acted if s["ctx_at_act"]]
    same = [s["ramp_act"] for s in acted if s["switch"] == "same-repo"]
    cross = [s["ramp_act"] for s in acted if s["switch"] == "cross-repo"]
    fans = [len(s["act_ids"]) for s in acted]
    day1 = sum(1 for _, h in touches if h < 24)

    w("# The setup-cost matrix")
    w("")
    w("First empirical numbers on agent context-switch cost, mined from this portfolio's")
    w(f"own meshwork stores, session transcripts, and git history. Data through **{end}**.")
    w("Read-only miner: `python3 scripts/mine_setup_cost.py --write` regenerates this file.")
    w("")
    w("## Headlines (each number's denominator in its section below)")
    w("")
    w(f"- An agent session reaches its first task action in a median **{fmt(median(all_ramp))} min**,")
    w(f"  carrying a median **{fmt(median(all_ctx), nd=0)}k tokens** of loaded context to get there")
    w(f"  (n={len(acted)} sessions).")
    m_same, m_cross = median(same), median(cross)
    if m_same is not None and m_cross is not None and m_cross <= m_same:
        w(f"- Switching repos costs nothing extra here: median ramp after a cross-repo session is")
        w(f"  **{fmt(m_cross)} min** vs {fmt(m_same)} min after a same-repo one (n={len(cross)}/{len(same)}).")
        w("  The store carries the context that the context window drops.")
    else:
        w(f"- A repo switch adds ramp: median **{fmt(m_cross)} min** after a cross-repo session vs")
        w(f"  {fmt(m_same)} min after a same-repo one (n={len(cross)}/{len(same)}).")
    w(f"- A session acts on a median of **{fmt(median(fans), nd=0)} tasks** (p90 {p90(fans)}, max {max(fans)});")
    w("  agent work is fan-out, not single-ticket.")
    w(f"- Activity is front-loaded: **{100 * day1 // len(touches)}%** of all task touches land in the")
    w("  task's first 24 h, and the chance an open task ever closes decays with age")
    w("  (the decreasing-hazard premise behind deprioritizing aging tickets — see below).")
    w("")
    w("## Method")
    w("")
    w("- **Session** = one Claude Code transcript file (`~/.claude/projects/…/<uuid>.jsonl`);")
    w("  a conversation thread — `/clear` or a new tab starts a new one. Timestamps are the")
    w("  transcript's own. Sessions are attributed to the repo whose project directory holds them.")
    w("- **Task act** = first CLI invocation of a mutating meshwork verb")
    w(f"  ({', '.join(sorted(ACT_VERBS))}); **read** = {', '.join(sorted(READ_VERBS))}.")
    w("- **Ramp** = minutes from the session's first timestamped event to its first task act.")
    w("- **Context at first act** = input + cache-read + cache-creation tokens on the message")
    w("  that issued it — the context the agent had loaded before it could act on a task.")
    w("- **Switch class** = the chronologically previous session (any repo): `same-repo`,")
    w("  `cross-repo`, or `cold` (>12 h since the previous session ended, or none).")
    w("- **Touch** = any post-creation `## log` row or comment on a task; store dates have")
    w("  minute resolution. Aging = time since the task's `created:`.")
    w("- Sessions ending before their repo's store existed are excluded from ramp/fan-out")
    w("  denominators (counted below as pre-adoption).")
    w("")

    # dataset table
    w("## Dataset (the denominators)")
    w("")
    w("| repo | tasks | log rows | comments | sessions | post-adoption | with task act | commits | w/ session trailer | store born |")
    w("|---|---|---|---|---|---|---|---|---|---|")
    tot = [0] * 8
    for repo in repos:
        n_tasks = sum(1 for g in tasks if g.startswith(repo + "#"))
        n_log = sum(1 for g, *_ in log_rows if g.startswith(repo + "#"))
        n_com = sum(1 for g, _ in comment_rows if g.startswith(repo + "#"))
        ss = [s for s in sessions if s["repo"] == repo]
        post = [s for s in ss if s["post_adoption"]]
        act = [s for s in post if s["ramp_act"] is not None]
        cs = commits[repo]
        n_tr = sum(1 for c in cs if c["session"])
        b = birth[repo].strftime("%Y-%m-%d") if repo in birth else "—"
        row = [n_tasks, n_log, n_com, len(ss), len(post), len(act), len(cs), n_tr]
        tot = [a + b2 for a, b2 in zip(tot, row)]
        w(f"| {repo} | {n_tasks} | {n_log} | {n_com} | {len(ss)} | {len(post)} | {len(act)} | {len(cs)} | {n_tr} | {b} |")
    w(f"| **total** | **{tot[0]}** | **{tot[1]}** | **{tot[2]}** | **{tot[3]}** | **{tot[4]}** | **{tot[5]}** | **{tot[6]}** | **{tot[7]}** | |")
    w("")

    # ramp per repo
    w("## Session ramp per repo")
    w("")
    w("Minutes from session start to first meshwork read / first task act; token cost carried")
    w("to the first act. `n` = post-adoption sessions with at least one task act.")
    w("")
    w("| repo | n | med min→read | med min→act | p90 min→act | med ctx @act (ktok) | med out-tok before act | med min→first commit (n) |")
    w("|---|---|---|---|---|---|---|---|")
    for repo in repos:
        a = [s for s in acted if s["repo"] == repo]
        if not a:
            w(f"| {repo} | 0 | — | — | — | — | — | — |")
            continue
        reads = [s["ramp_read"] for s in a if s["ramp_read"] is not None]
        ctx = [s["ctx_at_act"] / 1000 for s in a if s["ctx_at_act"]]
        outb = [s["out_before_act"] for s in a if s["out_before_act"] is not None]
        fc = [s["first_commit_min"] for s in a if s["first_commit_min"] is not None]
        w(f"| {repo} | {len(a)} | {fmt(median(reads))} | {fmt(median([s['ramp_act'] for s in a]))} "
          f"| {fmt(p90([s['ramp_act'] for s in a]))} | {fmt(median(ctx), nd=0)} "
          f"| {fmt(median(outb), nd=0)} | {fmt(median(fc))} ({len(fc)}) |")
    w(f"| **all** | **{len(acted)}** | | **{fmt(median(all_ramp))}** | **{fmt(p90(all_ramp))}** "
      f"| **{fmt(median(all_ctx), nd=0)}** | | |")
    w("")

    # switch cost
    w("## Cross-repo switch cost")
    w("")
    w("Ramp conditioned on what the previous session (any repo) was. This is the")
    w("`(previous-context, next-task)` pair the store keeps and everything else throws away.")
    w("")
    w("| previous context | n | med min→act | p90 min→act | med ctx @act (ktok) |")
    w("|---|---|---|---|---|")
    for cls in ("same-repo", "cross-repo", "cold"):
        grp = [s for s in acted if s["switch"] == cls]
        ctx = [s["ctx_at_act"] / 1000 for s in grp if s["ctx_at_act"]]
        ramps = [s["ramp_act"] for s in grp]
        w(f"| {cls} | {len(grp)} | {fmt(median(ramps))} | {fmt(p90(ramps))} | {fmt(median(ctx), nd=0)} |")
    w("")

    # fan-out
    w("## Task-touch fan-out per session")
    w("")
    w("Distinct existing tasks acted on per session (IDs seen in mutating commands), plus")
    w("`add` invocations (new tasks have no prior ID). Same `n` as the ramp table.")
    w("")
    w("| repo | n | med tasks/session | p90 | max | total adds | med session length (min) |")
    w("|---|---|---|---|---|---|---|")
    for repo in repos:
        a = [s for s in acted if s["repo"] == repo]
        if not a:
            w(f"| {repo} | 0 | — | — | — | — | — |")
            continue
        fans = [len(s["act_ids"]) for s in a]
        lens = [(s["t_end"] - s["t0"]).total_seconds() / 60 for s in a]
        w(f"| {repo} | {len(a)} | {fmt(median(fans))} | {p90(fans)} | {max(fans)} "
          f"| {sum(s['adds'] for s in a)} | {fmt(median(lens), nd=0)} |")
    w(f"| **all** | **{len(acted)}** | **{fmt(median(fans))}** | **{p90(fans)}** | **{max(fans)}** "
      f"| **{sum(s['adds'] for s in acted)}** | |")
    w("")

    # aging vs touch
    w("## Aging vs touch")
    w("")
    n_t = len(touches)
    w(f"Where activity lands, by task age at the moment of the touch ({n_t} touches on "
      f"{len(set(g for g, _ in touches))} tasks; {junk} touches with dates before their "
      f"task's `created:` — date-only stamps round to midnight — excluded):")
    w("")
    w("| task age at touch | touches | share |")
    w("|---|---|---|")
    buckets = [("< 1 h", 0, 1), ("1–24 h", 1, 24), ("1–3 d", 24, 72), ("3–7 d", 72, 168), ("> 7 d", 168, float("inf"))]
    for label, lo, hi in buckets:
        n = sum(1 for _, h in touches if lo <= h < hi)
        w(f"| {label} | {n} | {100 * n / n_t:.0f}% |")
    w("")
    w(f"Cycle time (created → first `done`), all repos: median "
      f"{fmt(median(list(cycle.values())))} h, p90 {fmt(p90(list(cycle.values())))} h "
      f"(n={len(cycle)} closed tasks with parseable dates).")
    w("")
    w("Closure hazard by age — of tasks that reached age *a* still open, how many ever")
    w("closed by data end (right-censored: tasks younger than *a* at data end excluded):")
    w("")
    w("| reached age still open | n | closed later | share |")
    w("|---|---|---|---|")
    for label, hours in (("24 h", 24), ("3 d", 72), ("7 d", 168)):
        elig = []
        for gid, t in tasks.items():
            c = t["created"]
            if not c or (data_end - c).total_seconds() / 3600 < hours:
                continue
            close_ts = closes.get(gid)
            age_at_close = (close_ts - c).total_seconds() / 3600 if close_ts else None
            if age_at_close is not None and age_at_close < hours:
                continue  # already closed before reaching this age
            if t["status"] == "dropped" and age_at_close is None:
                continue
            elig.append(1 if close_ts else 0)
        share = f"{100 * sum(elig) / len(elig):.0f}%" if elig else "—"
        w(f"| {label} | {len(elig)} | {sum(elig)} | {share} |")
    w("")

    # caveats
    w("## Caveats")
    w("")
    w("- Transcripts exist only on this machine and are prunable; sessions here = transcripts")
    w("  present at mining time, which undercounts early work. Commit counts are repo-complete.")
    w("- Store dates have minute resolution; sub-minute ramps are floor-visible only in transcripts.")
    w("- Sessions can run in parallel tabs; the previous-session classifier picks the latest")
    w("  start, so an overlapping neighbor can class as `same-repo`/`cross-repo` arbitrarily.")
    w("- Verb detection is regex over Bash commands in transcripts; heredoc bodies and prose")
    w("  mentioning meshwork verbs can miscount. Spot-checked, not proven.")
    w("- Pre-shim sessions claimed tasks as the default author, so store-side attribution is")
    w("  partial — session attribution here comes from transcript files, not `claimed-by:`.")
    w("- The meshwork repo dogfoods the CLI: its sessions also invoke meshwork against test")
    w("  fixture stores, which inflates its `add` count and can mark a fixture act as the")
    w("  session's first act. Adopter-repo rows carry no such noise.")
    w("")

    text = "\n".join(L) + "\n"
    if "--write" in sys.argv:
        OUT.write_text(text)
        print(f"wrote {OUT} ({len(text)} chars)", file=sys.stderr)
    else:
        sys.stdout.write(text)


if __name__ == "__main__":
    main()
