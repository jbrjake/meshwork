#!/usr/bin/env python3
"""Validation harness and baseline for docs/PROPOSAL-analytics.md — read-only.

The proposed analytics layer is a set of SQL views over the six-table projection. This script
holds every view as a CTE body, assembles each probe as `WITH RECURSIVE <every view> SELECT …`,
and runs it through per-repo `meshwork q --json` from each registered store's own checkout —
never a `portfolio` verb (those autoprune sequence.md as a side effect of reading). It prints
the pulse row per store, a probe per public view, and the pooled hazard and backlog-day tables.

    python3 scripts/mine_views.py            # human tables
    python3 scripts/mine_views.py --json     # one JSON document
    python3 scripts/mine_views.py --check    # exit 0 iff every probe on every present store exits 0
                                             # and FORMAT-views.sql is byte-equal to --sql
    python3 scripts/mine_views.py --sql      # the views as CREATE VIEW statements — the text of
                                             # FORMAT-views.sql, which the binary registers verbatim

`clock` is a TABLE in the binary (one row: now, today, source, window_days — FORMAT.md §Views);
this script emulates it as a CTE from MESHWORK_TODAY (either conforming stamp form; unset =
the wall clock) and MESHWORK_WINDOW_DAYS, so `--sql` omits it.

Env: MESHWORK_BIN (default target/release/meshwork, else target/debug), MESHWORK_CODE_ROOT
(default ~/Documents/code), MESHWORK_REPOS (comma list; default: the registry's names from the
portfolio repo's repos.toml when present, else a built-in list), MESHWORK_WINDOW_DAYS (default 7),
MESHWORK_TODAY (the emulated clock).
"""
import json
import os
import subprocess
import sys
import textwrap
import time
from collections import OrderedDict, defaultdict

ROOT = os.path.expanduser(os.environ.get("MESHWORK_CODE_ROOT", "~/Documents/code"))
_HERE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BIN = os.environ.get("MESHWORK_BIN") or next(
    (p for p in (os.path.join(_HERE, "target", "release", "meshwork"),
                 os.path.join(_HERE, "target", "debug", "meshwork")) if os.path.exists(p)),
    os.path.join(_HERE, "target", "debug", "meshwork"))
WINDOW = int(os.environ.get("MESHWORK_WINDOW_DAYS", "7"))
FALLBACK_REPOS = ["meshwork", "sazed", "leras", "marasi", "tensoon", "oreseur", "wyndam",
                  "marasi-applied-r-and-d", "portfolio"]

# The stamp guard: FORMAT.md's two conforming forms parse; anything else projects NULL.
# DataFusion's to_timestamp errors on a non-date token, so the guard is mandatory.
TS = ("CASE WHEN regexp_like({x}, '^\\d{{4}}-\\d{{2}}-\\d{{2}}(T\\d{{2}}:\\d{{2}}Z)?$') "
      "THEN to_timestamp({x}, '%Y-%m-%dT%H:%MZ', '%Y-%m-%d') END")


def ts(x):
    return TS.format(x=x)


V = OrderedDict()

# ---- layer 0: clock (a MemTable honouring MESHWORK_TODAY in the binary; emulated here) ----
STAMP = (os.environ.get("MESHWORK_TODAY") or "").strip()
_NOW = ts(f"'{STAMP}'") if STAMP else "now()"
V["clock"] = (f"SELECT {_NOW} AS now, CAST({_NOW} AS DATE) AS today, "
              f"'{'override' if STAMP else 'system'}' AS source, {WINDOW} AS window_days")
# Interval × integer does not plan in DataFusion 51, so the window is unix-second arithmetic.
V["p_win"] = "SELECT to_timestamp(to_unixtime(c.now) - c.window_days * 86400) AS since, c.now FROM clock c"

# ---- layer 1: events — the unified activity stream ----
V["events"] = f"""
SELECT t.gid, t.repo, 'created' AS kind, t.created AS stamp, {ts('t.created')} AS at,
       CAST(NULL AS VARCHAR) AS actor, CAST(NULL AS VARCHAR) AS from_status, CAST(NULL AS VARCHAR) AS to_status,
       CAST(NULL AS VARCHAR) AS note, 0 AS ord
FROM tasks t WHERE t.status <> 'invalid' AND t.created IS NOT NULL
UNION ALL
SELECT l.gid, t.repo,
       CASE WHEN l.to_status IS NOT NULL THEN 'transition'
            WHEN l.note LIKE 'close attempt%' THEN 'close-attempt'
            ELSE 'note' END AS kind,
       l.date AS stamp, {ts('l.date')} AS at,
       CASE WHEN l.note LIKE 'claimed by %' THEN substr(l.note, 12) END AS actor,
       l.from_status, l.to_status, l.note, l.ord
FROM log l JOIN tasks t ON t.gid = l.gid
UNION ALL
SELECT c.gid, t.repo, 'comment' AS kind, c.date AS stamp, {ts('c.date')} AS at,
       c.author AS actor, CAST(NULL AS VARCHAR) AS from_status, CAST(NULL AS VARCHAR) AS to_status, c.text AS note, c.ord
FROM comments c JOIN tasks t ON t.gid = c.gid
"""

# ---- layer 2: spans — time in each state ----
V["spans"] = """
SELECT gid, repo, state, entered, left_at, next_state,
       (to_unixtime(coalesce(left_at, c.now)) - to_unixtime(entered)) / 3600.0 AS hours
FROM (SELECT gid, repo, to_status AS state, at AS entered,
             lead(at) OVER (PARTITION BY gid ORDER BY ord) AS left_at,
             lead(to_status) OVER (PARTITION BY gid ORDER BY ord) AS next_state
      FROM events WHERE kind = 'transition' AND at IS NOT NULL) s
CROSS JOIN clock c
"""

# ---- layer 2: facts — one row per task, every lifecycle scalar ----
V["f_tr"] = """
SELECT gid,
       min(at) FILTER (WHERE to_status = 'doing') AS first_doing,
       max(at) FILTER (WHERE to_status = 'doing') AS last_doing,
       max(at) FILTER (WHERE to_status = 'done') AS done_at,
       max(at) FILTER (WHERE to_status IN ('done','dropped')) AS terminal_at,
       max(at) FILTER (WHERE kind = 'transition') AS last_transition,
       count(*) FILTER (WHERE to_status = 'blocked') AS blocks,
       count(*) FILTER (WHERE to_status = 'open' AND from_status IS NOT NULL) AS reopens,
       count(*) FILTER (WHERE kind = 'close-attempt') AS close_attempts,
       count(*) FILTER (WHERE kind = 'transition' AND actor IS NOT NULL) AS claims,
       count(*) FILTER (WHERE kind = 'comment') AS comments,
       count(DISTINCT actor) AS actors,
       max(at) AS last_activity,
       max(stamp) AS last_stamp,
       count(*) FILTER (WHERE kind <> 'created') AS touches
FROM events GROUP BY gid
"""
V["f_bl"] = "SELECT gid, sum(hours) AS blocked_h FROM spans WHERE state = 'blocked' GROUP BY gid"
V["f_dg"] = "SELECT gid, sum(hours) AS doing_h FROM spans WHERE state = 'doing' GROUP BY gid"
# The closure convention: done means CURRENT status — a reopened task has no done_at, no
# terminal_at, no cycle; its earlier →done line still counts as a transition and a touch.
V["facts"] = f"""
SELECT t.gid, t.repo, t.id, t.title, t.status, t.category, t.seq,
       t.status IN ('open','doing','blocked') AS live,
       {ts('t.created')} AS created_at,
       tr.first_doing, tr.last_doing,
       CASE WHEN t.status = 'done' THEN tr.done_at END AS done_at,
       CASE WHEN t.status IN ('done','dropped') THEN tr.terminal_at END AS terminal_at,
       tr.last_transition,
       (to_unixtime(coalesce(CASE WHEN t.status IN ('done','dropped') THEN tr.terminal_at END, c.now))
        - to_unixtime({ts('t.created')})) / 3600.0 AS age_h,
       (to_unixtime(tr.first_doing) - to_unixtime({ts('t.created')})) / 3600.0 AS queue_h,
       CASE WHEN t.status = 'done' AND tr.first_doing <= tr.done_at
            THEN (to_unixtime(tr.done_at) - to_unixtime(tr.first_doing)) / 3600.0 END AS service_h,
       CASE WHEN t.status = 'done'
            THEN (to_unixtime(tr.done_at) - to_unixtime({ts('t.created')})) / 3600.0 END AS cycle_h,
       coalesce(bl.blocked_h, 0) AS blocked_h,
       coalesce(dg.doing_h, 0) AS doing_h,
       tr.last_activity, tr.last_stamp,
       (to_unixtime(c.now) - to_unixtime(coalesce(tr.last_activity, {ts('t.created')}))) / 3600.0 AS idle_h,
       coalesce(tr.touches, 0) AS touches, coalesce(tr.comments, 0) AS comments,
       coalesce(tr.actors, 0) AS actors, coalesce(tr.claims, 0) AS claims,
       coalesce(tr.close_attempts, 0) AS close_attempts, coalesce(tr.reopens, 0) AS reopens,
       coalesce(tr.blocks, 0) AS blocks,
       t.verify IS NOT NULL AND trim(t.verify) <> '' AS has_verify,
       t.category IS NOT NULL AS has_category,
       t.seq IS NOT NULL AS has_seq,
       t.waived IS NOT NULL AS waived,
       t.handoff IS NOT NULL AS has_handoff,
       t.addressed_to IS NOT NULL AS is_ask,
       t.claimed_by
FROM tasks t
JOIN repos rp ON rp.repo = t.repo
LEFT JOIN f_tr tr ON tr.gid = t.gid
LEFT JOIN f_bl bl ON bl.gid = t.gid
LEFT JOIN f_dg dg ON dg.gid = t.gid
CROSS JOIN clock c
WHERE t.status <> 'invalid'
"""

# ---- layer 2: graph — structure over live needs ----
V["g_live"] = "SELECT gid, repo, coalesce(seq, 999999) AS place FROM tasks WHERE status IN ('open','doing','blocked')"
V["g_ln"] = """
SELECT e.src_gid, e.dst_gid FROM edges e
JOIN g_live a ON a.gid = e.src_gid JOIN g_live b ON b.gid = e.dst_gid
WHERE e.kind = 'needs'
"""
V["g_up"] = """
SELECT dst_gid AS pre, src_gid AS dep, 1 AS d FROM g_ln
UNION ALL
SELECT g_up.pre, g_ln.src_gid, g_up.d + 1 FROM g_up JOIN g_ln ON g_ln.dst_gid = g_up.dep WHERE g_up.d < 64
"""
V["g_unl"] = "SELECT pre AS gid, count(DISTINCT dep) AS unlock, max(d) AS depth FROM g_up GROUP BY pre"
V["g_inh"] = """
SELECT u.pre AS gid, min(l.place) AS inherit
FROM g_up u JOIN g_live p ON p.gid = u.pre JOIN g_live l ON l.gid = u.dep AND l.repo = p.repo
GROUP BY u.pre
"""
V["g_nodes"] = "SELECT src_gid AS n FROM g_ln UNION SELECT dst_gid AS n FROM g_ln"
V["g_und"] = """
SELECT src_gid AS a, dst_gid AS b FROM g_ln
UNION ALL SELECT dst_gid AS a, src_gid AS b FROM g_ln
UNION ALL SELECT n AS a, n AS b FROM g_nodes
"""
# Synchronous min-label propagation; the recursive term emits rows only while a label changed,
# so the recursion converges in diameter+1 rounds instead of paying 64 iterations of planning.
V["g_lab"] = """
SELECT n AS node, n AS lane, 0 AS i, true AS changed FROM g_nodes
UNION ALL
SELECT node, lane, i, changed FROM (
  SELECT node, lane, i, changed, sum(CASE WHEN changed THEN 1 ELSE 0 END) OVER () AS n_changed
  FROM (SELECT g_und.a AS node, min(g_lab.lane) AS lane, g_lab.i + 1 AS i,
               min(g_lab.lane) < min(CASE WHEN g_und.a = g_und.b THEN g_lab.lane END) AS changed
        FROM g_lab JOIN g_und ON g_und.b = g_lab.node WHERE g_lab.i < 64
        GROUP BY g_und.a, g_lab.i) step) marked
WHERE n_changed > 0
"""
V["g_lanes"] = "SELECT node AS gid, min(lane) AS lane FROM g_lab GROUP BY node"
V["g_lsize"] = "SELECT lane, count(*) AS lane_size FROM g_lanes GROUP BY lane"
V["g_needs_open"] = """
SELECT e.src_gid AS gid, count(*) AS needs_open,
       count(*) FILTER (WHERE d.gid IS NULL) AS needs_unresolved,
       count(*) FILTER (WHERE d.repo IS NOT NULL AND d.repo <> t.repo) AS needs_open_foreign
FROM edges e JOIN tasks t ON t.gid = e.src_gid LEFT JOIN tasks d ON d.gid = e.dst_gid
WHERE e.kind = 'needs' AND (d.status IS NULL OR d.status NOT IN ('done','dropped'))
GROUP BY e.src_gid
"""
V["g_needed_by"] = """
SELECT e.dst_gid AS gid, count(*) AS needed_by_open,
       count(*) FILTER (WHERE s.repo <> t.repo) AS needed_by_foreign
FROM edges e JOIN tasks s ON s.gid = e.src_gid JOIN tasks t ON t.gid = e.dst_gid
WHERE e.kind = 'needs' AND s.status IN ('open','doing','blocked')
GROUP BY e.dst_gid
"""
V["g_children"] = """
SELECT e.dst_gid AS gid, count(*) AS live_children FROM edges e JOIN tasks ch ON ch.gid = e.src_gid
WHERE e.kind = 'parent' AND ch.status IN ('open','doing','blocked') GROUP BY e.dst_gid
"""
V["g_spawned"] = "SELECT dst_gid AS gid, count(*) AS discovered FROM edges WHERE kind = 'discovered-from' GROUP BY dst_gid"
V["graph"] = """
SELECT t.gid, t.repo, t.id, t.status, t.seq,
       coalesce(u.unlock, 0) AS unlock, coalesce(u.depth, 0) AS depth,
       coalesce(t.seq, 999999) AS place,
       coalesce(i.inherit, coalesce(t.seq, 999999)) AS inherit,
       coalesce(i.inherit < coalesce(t.seq, 999999), false) AS needs_behind,
       ln.lane, coalesce(ls.lane_size, 1) AS lane_size,
       coalesce(no.needs_open, 0) AS needs_open,
       coalesce(no.needs_unresolved, 0) AS needs_unresolved,
       coalesce(no.needs_open_foreign, 0) AS needs_open_foreign,
       coalesce(nb.needed_by_open, 0) AS needed_by_open,
       coalesce(nb.needed_by_foreign, 0) AS needed_by_foreign,
       coalesce(ch.live_children, 0) AS live_children,
       coalesce(sp.discovered, 0) AS discovered,
       t.status = 'open' AND coalesce(no.needs_open, 0) = 0 AND coalesce(ch.live_children, 0) = 0 AS ready
FROM tasks t
LEFT JOIN g_unl u ON u.gid = t.gid
LEFT JOIN g_inh i ON i.gid = t.gid
LEFT JOIN g_lanes ln ON ln.gid = t.gid
LEFT JOIN g_lsize ls ON ls.lane = ln.lane
LEFT JOIN g_needs_open no ON no.gid = t.gid
LEFT JOIN g_needed_by nb ON nb.gid = t.gid
LEFT JOIN g_children ch ON ch.gid = t.gid
LEFT JOIN g_spawned sp ON sp.gid = t.gid
JOIN repos rp ON rp.repo = t.repo
WHERE t.status <> 'invalid'
"""

# ---- layer 2: asks — obligations across repos ----
# State, not the suppression rule: how many answers are open, how many done; `answered` needs a
# DONE answer and `unanswered` is a live ask with none — an open answer keeps the ask unanswered.
V["a_ans"] = """
SELECT e.dst_gid AS ask_gid, min(x.gid) AS answer_gid,
       count(*) FILTER (WHERE x.status IN ('open','doing','blocked')) AS answer_open,
       count(*) FILTER (WHERE x.status = 'done') AS answer_done,
       min(fx.created_at) AS answered_at, max(fx.done_at) AS answer_done_at
FROM edges e JOIN tasks x ON x.gid = e.src_gid JOIN facts fx ON fx.gid = x.gid
WHERE e.kind = 'answers' AND x.status <> 'dropped'
GROUP BY e.dst_gid
"""
V["asks"] = """
SELECT a.gid, a.repo AS from_repo, split_part(a.addressed_to, '#', 1) AS to_repo, a.addressed_to AS to_ref,
       a.title, a.status, f.created_at, f.age_h, f.idle_h, f.seq,
       ans.answer_gid, ax.status AS answer_status,
       coalesce(ans.answer_open, 0) AS answer_open, coalesce(ans.answer_done, 0) AS answer_done,
       ans.answered_at, ans.answer_done_at,
       (to_unixtime(ans.answered_at) - to_unixtime(f.created_at)) / 3600.0 AS response_h,
       coalesce(ans.answer_done, 0) > 0 AS answered,
       a.status NOT IN ('done','dropped') AND coalesce(ans.answer_done, 0) = 0 AS unanswered
FROM tasks a JOIN facts f ON f.gid = a.gid
LEFT JOIN a_ans ans ON ans.ask_gid = a.gid
LEFT JOIN tasks ax ON ax.gid = ans.answer_gid
WHERE a.addressed_to IS NOT NULL
"""

# ---- layer 2: mentions — ids named in ANY text field, resolved to current status ----
V["m_tok"] = """
SELECT gid, repo, 'handoff' AS field, CAST(NULL AS VARCHAR) AS stamp,
       unnest(string_to_array(regexp_replace(handoff, '[^a-z0-9#-]+', ' ', 'g'), ' ')) AS tok
FROM tasks WHERE handoff IS NOT NULL
UNION ALL
SELECT gid, repo, 'body' AS field, CAST(NULL AS VARCHAR) AS stamp,
       unnest(string_to_array(regexp_replace(body, '[^a-z0-9#-]+', ' ', 'g'), ' ')) AS tok
FROM tasks WHERE body IS NOT NULL AND body <> ''
UNION ALL
SELECT c.gid, t.repo, 'comment' AS field, c.date AS stamp,
       unnest(string_to_array(regexp_replace(c.text, '[^a-z0-9#-]+', ' ', 'g'), ' ')) AS tok
FROM comments c JOIN tasks t ON t.gid = c.gid
UNION ALL
SELECT l.gid, t.repo, 'log' AS field, l.date AS stamp,
       unnest(string_to_array(regexp_replace(l.note, '[^a-z0-9#-]+', ' ', 'g'), ' ')) AS tok
FROM log l JOIN tasks t ON t.gid = l.gid WHERE l.note IS NOT NULL
"""
# Tokens normalize to repo#id first so the resolution is an equality join, never a nested loop.
V["m_pairs"] = f"""
SELECT k.gid AS src_gid, k.repo AS src_repo, k.field, r.gid AS ref_gid,
       min({ts('k.stamp')}) AS first_at, count(*) AS times
FROM (SELECT gid, repo, field, stamp,
             CASE WHEN strpos(tok, '#') > 0 THEN tok ELSE repo || '#' || tok END AS ref
      FROM m_tok WHERE tok <> '') k
JOIN tasks r ON r.gid = k.ref
WHERE r.gid <> k.gid
GROUP BY k.gid, k.repo, k.field, r.gid
"""
V["m_edge"] = "SELECT DISTINCT a, b FROM (SELECT src_gid AS a, dst_gid AS b FROM edges UNION ALL SELECT dst_gid AS a, src_gid AS b FROM edges)"
V["mentions"] = """
SELECT p.src_gid, p.src_repo, p.field, p.ref_gid, p.times, p.first_at,
       r.status AS ref_status, fr.terminal_at AS ref_terminal_at, fs.last_activity AS src_last_activity,
       coalesce(fr.terminal_at > fs.last_activity, false) AS moved_since_activity,
       eb.a IS NOT NULL AS edge_backed
FROM m_pairs p JOIN tasks r ON r.gid = p.ref_gid
JOIN facts fr ON fr.gid = r.gid JOIN facts fs ON fs.gid = p.src_gid
LEFT JOIN m_edge eb ON eb.a = p.src_gid AND eb.b = p.ref_gid
"""

# ---- layer 2: lineage — spawn trees (discovered-from) and umbrellas (parent), with growth ----
V["li_df"] = "SELECT src_gid AS child, dst_gid AS origin FROM edges WHERE kind = 'discovered-from'"
V["li_sp"] = """
SELECT origin, child, 1 AS d FROM li_df
UNION ALL
SELECT li_sp.origin, li_df.child, li_sp.d + 1 FROM li_sp JOIN li_df ON li_df.origin = li_sp.child WHERE li_sp.d < 64
"""
V["li_spawn"] = """
SELECT s.origin AS gid,
       count(*) FILTER (WHERE s.d = 1) AS spawned_direct,
       count(*) AS spawned_total,
       count(*) FILTER (WHERE f.live) AS spawned_live,
       max(s.d) AS spawn_depth,
       max(f.created_at) AS last_spawn_at,
       count(*) FILTER (WHERE f.created_at >= w.since) AS spawned_w,
       count(*) FILTER (WHERE f.status = 'done') AS spawned_done
FROM li_sp s JOIN facts f ON f.gid = s.child CROSS JOIN p_win w
GROUP BY s.origin
"""
V["li_pa"] = "SELECT src_gid AS child, dst_gid AS parent FROM edges WHERE kind = 'parent'"
V["li_um"] = """
SELECT parent AS root, child, 1 AS d FROM li_pa
UNION ALL
SELECT li_um.root, li_pa.child, li_um.d + 1 FROM li_um JOIN li_pa ON li_pa.parent = li_um.child WHERE li_um.d < 64
"""
V["li_umbrella"] = """
SELECT u.root AS gid,
       count(*) FILTER (WHERE u.d = 1) AS children_direct,
       count(*) AS descendants_total,
       count(*) FILTER (WHERE f.live) AS descendants_live,
       max(u.d) AS umbrella_depth,
       max(f.created_at) AS last_child_at,
       count(*) FILTER (WHERE f.created_at >= w.since) AS children_w,
       count(*) FILTER (WHERE f.status = 'done') AS descendants_done
FROM li_um u JOIN facts f ON f.gid = u.child CROSS JOIN p_win w
GROUP BY u.root
"""
V["li_in"] = """
SELECT dst_gid AS gid,
       count(*) FILTER (WHERE kind = 'needs') AS needs_in,
       count(*) FILTER (WHERE kind = 'relates') AS relates_in,
       count(*) FILTER (WHERE kind = 'answers') AS answers_in,
       count(DISTINCT src_gid) AS referrers_edges
FROM edges GROUP BY dst_gid
"""
V["li_men"] = """
SELECT ref_gid AS gid, count(DISTINCT src_gid) AS mentioned_by,
       count(DISTINCT src_gid) FILTER (WHERE NOT edge_backed) AS mentioned_by_unlinked,
       count(DISTINCT src_gid) FILTER (WHERE field = 'comment' OR field = 'log') AS mentioned_in_talk
FROM mentions GROUP BY ref_gid
"""
V["lineage"] = """
SELECT t.gid, t.repo, t.id, t.status,
       coalesce(sp.spawned_direct, 0) AS spawned_direct, coalesce(sp.spawned_total, 0) AS spawned_total,
       coalesce(sp.spawned_live, 0) AS spawned_live, coalesce(sp.spawned_done, 0) AS spawned_done,
       coalesce(sp.spawn_depth, 0) AS spawn_depth, sp.last_spawn_at, coalesce(sp.spawned_w, 0) AS spawned_w,
       coalesce(um.children_direct, 0) AS children_direct, coalesce(um.descendants_total, 0) AS descendants_total,
       coalesce(um.descendants_live, 0) AS descendants_live, coalesce(um.descendants_done, 0) AS descendants_done,
       coalesce(um.umbrella_depth, 0) AS umbrella_depth, um.last_child_at, coalesce(um.children_w, 0) AS children_w,
       coalesce(li.needs_in, 0) AS needs_in, coalesce(li.relates_in, 0) AS relates_in, coalesce(li.answers_in, 0) AS answers_in,
       coalesce(li.referrers_edges, 0) AS referrers_edges,
       coalesce(mn.mentioned_by, 0) AS mentioned_by, coalesce(mn.mentioned_by_unlinked, 0) AS mentioned_by_unlinked,
       coalesce(mn.mentioned_in_talk, 0) AS mentioned_in_talk,
       coalesce(sp.spawned_total, 0) + coalesce(um.descendants_total, 0) + coalesce(li.referrers_edges, 0) + coalesce(mn.mentioned_by, 0) AS gravity
FROM tasks t
LEFT JOIN li_spawn sp ON sp.gid = t.gid
LEFT JOIN li_umbrella um ON um.gid = t.gid
LEFT JOIN li_in li ON li.gid = t.gid
LEFT JOIN li_men mn ON mn.gid = t.gid
JOIN repos rp ON rp.repo = t.repo
WHERE t.status <> 'invalid'
"""

# ---- layer 3: flow — the daily series (per repo), O(n) via daily buckets + window sums ----
V["fl_days"] = """
SELECT r.repo, d.day FROM repos r
CROSS JOIN (SELECT unnest(generate_series((SELECT CAST(min(created_at) AS DATE) FROM facts),
                                          (SELECT today FROM clock), INTERVAL '1 day')) AS day) d
"""
V["fl_ev"] = """
SELECT repo, CAST(created_at AS DATE) AS day, 1 AS filed, 0 AS done, 0 AS dropped, 0 AS doing_in, 0 AS doing_out
FROM facts WHERE created_at IS NOT NULL
UNION ALL
SELECT repo, CAST(terminal_at AS DATE) AS day, 0 AS filed, CASE WHEN status = 'done' THEN 1 ELSE 0 END AS done,
       CASE WHEN status = 'dropped' THEN 1 ELSE 0 END AS dropped, 0 AS doing_in, 0 AS doing_out
FROM facts WHERE terminal_at IS NOT NULL
UNION ALL
SELECT repo, CAST(entered AS DATE) AS day, 0 AS filed, 0 AS done, 0 AS dropped, 1 AS doing_in, 0 AS doing_out
FROM spans WHERE state = 'doing'
UNION ALL
SELECT repo, CAST(left_at AS DATE) AS day, 0 AS filed, 0 AS done, 0 AS dropped, 0 AS doing_in, 1 AS doing_out
FROM spans WHERE state = 'doing' AND left_at IS NOT NULL
"""
V["fl_day"] = """
SELECT repo, day, sum(filed) AS filed, sum(done) AS done, sum(dropped) AS dropped,
       sum(doing_in) AS doing_in, sum(doing_out) AS doing_out
FROM fl_ev GROUP BY repo, day
"""
V["flow"] = """
SELECT repo, day, filed, done, dropped,
       sum(filed) OVER w AS filed_cum, sum(done) OVER w AS done_cum, sum(dropped) OVER w AS dropped_cum,
       sum(filed - done - dropped) OVER w AS backlog,
       filed - done - dropped AS backlog_delta,
       sum(doing_in - doing_out) OVER w AS doing_eod
FROM (SELECT d.repo, d.day, coalesce(e.filed, 0) AS filed, coalesce(e.done, 0) AS done, coalesce(e.dropped, 0) AS dropped,
             coalesce(e.doing_in, 0) AS doing_in, coalesce(e.doing_out, 0) AS doing_out
      FROM fl_days d LEFT JOIN fl_day e ON e.repo = d.repo AND e.day = d.day) x
WINDOW w AS (PARTITION BY repo ORDER BY day)
"""

# ---- layer 3: hazard — discrete daily close hazard, competing risks, equalized follow-up ----
V["hz_life"] = """
SELECT gid, repo, status, age_h / 24.0 AS age_d,
       (to_unixtime(c.now) - to_unixtime(created_at)) / 86400.0 AS observed_d
FROM facts CROSS JOIN clock c WHERE created_at IS NOT NULL
"""
V["hazard"] = """
SELECT l.repo, b.age_d, count(*) AS at_risk,
       count(*) FILTER (WHERE l.status = 'done' AND l.age_d < b.age_d + 1) AS done_in,
       count(*) FILTER (WHERE l.status = 'dropped' AND l.age_d < b.age_d + 1) AS dropped_in
FROM (SELECT unnest(generate_series(0, 60)) AS age_d) b
JOIN hz_life l ON l.age_d >= b.age_d AND l.observed_d >= b.age_d + 1
GROUP BY l.repo, b.age_d
"""

# ---- layer 3: attention — who touched a live task since it last moved ----
V["attention"] = """
SELECT f.gid, f.repo, f.id, f.status, f.idle_h, f.actors,
       count(DISTINCT e.actor) AS touched_since_move,
       min(e.at) AS first_touch_since_move
FROM facts f LEFT JOIN events e ON e.gid = f.gid AND e.actor IS NOT NULL
     AND (f.last_transition IS NULL OR e.at > f.last_transition)
WHERE f.live
GROUP BY f.gid, f.repo, f.id, f.status, f.idle_h, f.actors
"""

# ---- layer 3: sessions — per self-professed identity ----
V["sessions"] = """
SELECT actor, repo, min(at) AS first_seen, max(at) AS last_seen,
       count(DISTINCT gid) AS tasks_touched,
       count(*) FILTER (WHERE kind = 'comment') AS comments,
       count(*) FILTER (WHERE kind = 'transition') AS claims
FROM events WHERE actor IS NOT NULL GROUP BY actor, repo
"""

# ---- layer 3: pulse — one row per repo, the weather ----
# Window closes follow the closure convention: a task closed in the window and reopened since
# is not a close, so done_w/dropped_w come from facts (current status), not from the log.
V["p_term"] = """
SELECT f.repo,
       count(*) FILTER (WHERE f.status = 'done' AND f.terminal_at >= w.since) AS done,
       count(*) FILTER (WHERE f.status = 'dropped' AND f.terminal_at >= w.since) AS dropped
FROM facts f CROSS JOIN p_win w GROUP BY f.repo
"""
V["p_ev"] = """
SELECT e.repo,
       count(*) FILTER (WHERE kind = 'created') AS filed,
       count(DISTINCT gid) FILTER (WHERE to_status = 'doing') AS started,
       count(*) FILTER (WHERE to_status = 'blocked') AS blocks,
       count(*) FILTER (WHERE to_status = 'open' AND from_status IS NOT NULL) AS reopens,
       count(*) FILTER (WHERE kind = 'close-attempt') AS close_attempts,
       count(*) FILTER (WHERE kind = 'comment') AS comments,
       count(DISTINCT actor) AS actors
FROM events e CROSS JOIN p_win w WHERE e.at >= w.since GROUP BY e.repo
"""
V["p_state"] = """
SELECT repo,
       count(*) FILTER (WHERE status = 'open') AS open_n,
       count(*) FILTER (WHERE status = 'doing') AS doing_n,
       count(*) FILTER (WHERE status = 'blocked') AS blocked_n,
       count(*) FILTER (WHERE status = 'done') AS done_n,
       count(*) FILTER (WHERE status = 'dropped') AS dropped_n,
       count(*) FILTER (WHERE status = 'doing' AND idle_h >= 72) AS doing_stale,
       round(median(age_h) FILTER (WHERE status = 'open') / 24.0, 1) AS open_age_med_d,
       count(*) FILTER (WHERE live AND age_h >= 14 * 24) AS past_triage,
       count(*) FILTER (WHERE live AND NOT has_verify) AS no_verify,
       count(*) FILTER (WHERE live AND NOT has_seq) AS no_seq,
       count(*) FILTER (WHERE live AND NOT has_category) AS no_category,
       count(*) FILTER (WHERE live AND has_seq AND idle_h >= 14 * 24) AS parked_in_place,
       count(*) FILTER (WHERE status = 'done' AND waived) AS waived_n
FROM facts GROUP BY repo
"""
V["p_graph"] = """
SELECT repo,
       count(*) FILTER (WHERE ready) AS ready_n,
       count(*) FILTER (WHERE unlock > 0) AS unlockers,
       count(DISTINCT lane) FILTER (WHERE lane_size > 1) AS lanes_multi,
       count(*) FILTER (WHERE needs_behind) AS needs_behind_n,
       count(*) FILTER (WHERE status IN ('open','doing','blocked') AND needs_open_foreign > 0) AS blocked_foreign,
       count(*) FILTER (WHERE status IN ('open','doing','blocked') AND needs_unresolved > 0) AS blocked_unresolved,
       count(*) FILTER (WHERE status IN ('open','doing','blocked') AND needed_by_foreign > 0) AS owed_foreign
FROM graph GROUP BY repo
"""
V["p_asks_in"] = """
SELECT to_repo AS repo, count(*) FILTER (WHERE unanswered) AS asks_in_open,
       round(max(age_h) FILTER (WHERE unanswered) / 24.0, 1) AS asks_in_oldest_d
FROM asks GROUP BY to_repo
"""
V["p_asks_out"] = """
SELECT from_repo AS repo, count(*) FILTER (WHERE unanswered) AS asks_out_open,
       round(max(age_h) FILTER (WHERE unanswered) / 24.0, 1) AS asks_out_oldest_d
FROM asks GROUP BY from_repo
"""
V["p_thrash"] = "SELECT repo, count(*) AS thrash_n FROM attention WHERE touched_since_move >= 2 GROUP BY repo"
V["p_refs"] = "SELECT src_repo AS repo, count(DISTINCT src_gid) AS handoff_stale_n FROM mentions WHERE field = 'handoff' AND ref_status IN ('done','dropped') GROUP BY src_repo"
V["p_lin"] = """
SELECT l.repo,
       count(*) FILTER (WHERE l.spawned_w > 0) AS spawners_w,
       max(l.spawned_w) AS spawned_w_max,
       count(*) FILTER (WHERE l.children_w > 0) AS umbrellas_w,
       sum(l.mentioned_by_unlinked) AS unlinked_mentions
FROM lineage l GROUP BY l.repo
"""
V["p_from"] = """
SELECT f.repo, count(DISTINCT f.gid) AS filed_from_w
FROM facts f JOIN edges e ON e.src_gid = f.gid AND e.kind = 'discovered-from' CROSS JOIN p_win w
WHERE f.created_at >= w.since GROUP BY f.repo
"""
V["p_implicit"] = """
SELECT m.src_repo AS repo, count(*) AS implicit_live_n
FROM mentions m JOIN facts a ON a.gid = m.src_gid JOIN facts b ON b.gid = m.ref_gid
WHERE NOT m.edge_backed AND a.live AND b.live GROUP BY m.src_repo
"""
V["pulse"] = """
SELECT r.repo,
       s.open_n, s.doing_n, s.blocked_n, s.done_n, s.dropped_n,
       coalesce(e.filed, 0) AS filed_w, coalesce(tm.done, 0) AS done_w, coalesce(tm.dropped, 0) AS dropped_w,
       coalesce(e.started, 0) AS started_w,
       coalesce(e.filed, 0) - coalesce(tm.done, 0) - coalesce(tm.dropped, 0) AS backlog_delta_w,
       s.doing_stale, s.open_age_med_d, s.past_triage, s.parked_in_place,
       coalesce(g.ready_n, 0) AS ready_n, coalesce(g.unlockers, 0) AS unlockers, coalesce(g.lanes_multi, 0) AS lanes_multi,
       coalesce(g.needs_behind_n, 0) AS needs_behind_n, coalesce(g.blocked_foreign, 0) AS blocked_foreign,
       coalesce(g.blocked_unresolved, 0) AS blocked_unresolved, coalesce(g.owed_foreign, 0) AS owed_foreign,
       coalesce(ai.asks_in_open, 0) AS asks_in_open, ai.asks_in_oldest_d,
       coalesce(ao.asks_out_open, 0) AS asks_out_open, ao.asks_out_oldest_d,
       coalesce(e.close_attempts, 0) AS close_attempts_w, coalesce(e.reopens, 0) AS reopens_w,
       coalesce(e.blocks, 0) AS blocks_w, s.waived_n, s.no_verify, s.no_seq, s.no_category,
       coalesce(e.comments, 0) AS comments_w, coalesce(e.actors, 0) AS actors_w,
       coalesce(th.thrash_n, 0) AS thrash_n, coalesce(rf.handoff_stale_n, 0) AS handoff_stale_n,
       coalesce(fr.filed_from_w, 0) AS filed_from_w, coalesce(li.spawners_w, 0) AS spawners_w,
       coalesce(li.spawned_w_max, 0) AS spawned_w_max, coalesce(li.umbrellas_w, 0) AS umbrellas_w,
       coalesce(li.unlinked_mentions, 0) AS unlinked_mentions, coalesce(im.implicit_live_n, 0) AS implicit_live_n
FROM repos r
LEFT JOIN p_state s ON s.repo = r.repo
LEFT JOIN p_ev e ON e.repo = r.repo
LEFT JOIN p_term tm ON tm.repo = r.repo
LEFT JOIN p_graph g ON g.repo = r.repo
LEFT JOIN p_asks_in ai ON ai.repo = r.repo
LEFT JOIN p_asks_out ao ON ao.repo = r.repo
LEFT JOIN p_thrash th ON th.repo = r.repo
LEFT JOIN p_refs rf ON rf.repo = r.repo
LEFT JOIN p_lin li ON li.repo = r.repo
LEFT JOIN p_from fr ON fr.repo = r.repo
LEFT JOIN p_implicit im ON im.repo = r.repo
"""

# One probe per public view; `pulse` is the baseline row.
PROBES = OrderedDict([
    ("pulse", "SELECT * FROM pulse"),
    ("facts", "SELECT count(*) AS n, count(*) FILTER (WHERE live) AS live, round(median(service_h), 2) AS svc_med_h, round(median(queue_h), 1) AS queue_med_h, round(median(cycle_h), 1) AS cycle_med_h, count(service_h) AS n_svc FROM facts"),
    ("spans", "SELECT state, count(*) AS n, round(median(hours), 2) AS med_h, round(approx_percentile_cont(hours, 0.9), 1) AS p90_h FROM spans WHERE state IN ('doing','blocked') GROUP BY state ORDER BY state"),
    ("graph", "SELECT id, status, unlock, depth, place, inherit, needs_behind, lane_size, ready FROM graph WHERE unlock > 0 ORDER BY unlock DESC, id LIMIT 5"),
    ("lanes", "SELECT lane, lane_size FROM (SELECT DISTINCT lane, lane_size FROM graph WHERE lane IS NOT NULL) ORDER BY lane_size DESC, lane LIMIT 5"),
    ("asks", "SELECT gid, to_repo, round(age_h / 24.0, 1) AS age_d FROM asks WHERE unanswered ORDER BY age_h DESC LIMIT 5"),
    ("mentions", "SELECT field, count(*) AS pairs, count(*) FILTER (WHERE NOT edge_backed) AS unlinked, count(*) FILTER (WHERE ref_status IN ('done','dropped')) AS to_terminal FROM mentions GROUP BY field ORDER BY pairs DESC"),
    ("lineage", "SELECT id, status, spawned_total, spawned_live, spawn_depth, spawned_w, descendants_total, descendants_live, mentioned_by, gravity FROM lineage ORDER BY gravity DESC, id LIMIT 5"),
    ("flow", "SELECT day, filed_cum, done_cum, backlog, filed, done, backlog_delta, doing_eod FROM flow ORDER BY day DESC LIMIT 3"),
    ("hazard", "SELECT age_d, at_risk, done_in, dropped_in FROM hazard ORDER BY age_d"),
    ("attention", "SELECT id, touched_since_move, round(idle_h / 24.0, 1) AS idle_d FROM attention ORDER BY touched_since_move DESC, idle_h DESC LIMIT 3"),
    ("sessions", "SELECT actor, tasks_touched, comments, claims FROM sessions ORDER BY tasks_touched DESC LIMIT 3"),
])
HAZARD_ROWS = (0, 1, 2, 3, 4, 5, 6, 7, 10, 14, 21, 28)


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
                if line.startswith("name") and "=" in line and line.split("=", 1)[1].count('"') >= 2:
                    v = line.split("=", 1)[1].split('"')[1]
                    if v:
                        names.append(v)
    return names or FALLBACK_REPOS


def assemble(select_sql):
    ctes = ",\n".join(f"{name} AS ({textwrap.dedent(body).strip()})" for name, body in V.items())
    return f"WITH RECURSIVE\n{ctes}\n{select_sql}"


def run(repo, select_sql):
    """(rows, error, seconds) for one probe against one store; rows as dicts."""
    cwd = os.path.join(ROOT, repo)
    t0 = time.time()
    r = subprocess.run([BIN, "--json", "q", assemble(select_sql)], cwd=cwd, capture_output=True, text=True)
    dt = time.time() - t0
    if r.returncode != 0:
        return None, (r.stderr.strip().splitlines() or ["(no stderr)"])[0][:300], dt
    d = json.loads(r.stdout)["data"]
    return [dict(zip(d["columns"], row)) for row in d["rows"]], None, dt


SQL_HEADER = """\
-- FORMAT-views.sql — the derived projection's reference SQL (FORMAT.md §Views).
-- Generated by `scripts/mine_views.py --sql`; the binary registers this text verbatim, in
-- order, after the six tables and the one-row `clock` table (now, today, source, window_days).
-- `clock` is a table, never a view; every duration and window is taken from it. Helper views
-- (short lowercase prefixes) are implementation; only the thirteen named in FORMAT.md are
-- contract. DataFusion 51 dialect: the stamp guard, `to_unixtime` window arithmetic, recursive
-- CTEs bounded at 64, and the converging label propagation are explained in FORMAT.md.
"""


# The views that name themselves: as CTEs the chain's WITH RECURSIVE covers them; as
# standalone views each carries its own, the CTE shadowing the view's name.
RECURSIVE = {"g_up", "g_lab", "li_sp", "li_um"}


def view_body(name, body):
    body = textwrap.dedent(body).strip()
    if name in RECURSIVE:
        return f"WITH RECURSIVE {name} AS (\n{body}\n) SELECT * FROM {name}"
    return body


def sql_text():
    return SQL_HEADER + "\n".join(f"CREATE VIEW {n} AS\n{view_body(n, b)};\n"
                                  for n, b in V.items() if n != "clock")


def main():
    if "--sql" in sys.argv:
        print(sql_text(), end="")
        return 0
    out, failures = {}, []
    haz, byday = defaultdict(lambda: [0, 0, 0]), defaultdict(int)
    for repo in repos():
        if not os.path.isdir(os.path.join(ROOT, repo, "docs", "meshwork")):
            continue
        out[repo] = {}
        for name, sql in PROBES.items():
            rows, err, dt = run(repo, sql)
            out[repo][name] = {"rows": rows, "error": err, "seconds": round(dt, 3)}
            if err:
                failures.append((repo, name, err))
        for r in out[repo]["hazard"]["rows"] or []:
            h = haz[r["age_d"]]
            h[0] += r["at_risk"]
            h[1] += r["done_in"]
            h[2] += r["dropped_in"]
        rows, err, _ = run(repo, "SELECT day, backlog_delta FROM flow")
        for r in rows or []:
            byday[r["day"]] += r["backlog_delta"] or 0
    pooled = {"hazard": [{"age_d": a, "at_risk": v[0], "done_in": v[1], "dropped_in": v[2],
                          "h_done": round(v[1] / v[0], 3), "h_drop": round(v[2] / v[0], 3)}
                         for a, v in sorted(haz.items()) if v[0]],
              "backlog_days": {"widened": sum(1 for v in byday.values() if v > 0),
                               "narrowed": sum(1 for v in byday.values() if v < 0),
                               "flat": sum(1 for v in byday.values() if v == 0), "days": len(byday)}}
    doc = {"as_of": time.strftime("%Y-%m-%dT%H:%MZ", time.gmtime()), "binary": BIN, "window_days": WINDOW,
           "repos": out, "pooled": pooled, "failures": failures}
    if "--json" in sys.argv:
        print(json.dumps(doc, indent=1, default=str))
    else:
        print(f"analytics baseline as of {doc['as_of']} — {len(out)} stores, window {WINDOW} d, binary {BIN}")
        for repo, probes in out.items():
            p = (probes["pulse"]["rows"] or [{}])[0]
            print(f"\n== {repo}: " + " · ".join(f"{k} {v}" for k, v in p.items() if k != "repo"))
            for name, res in probes.items():
                if name == "pulse":
                    continue
                if res["error"]:
                    print(f"   {name}: ERROR {res['error']}")
                    continue
                print(f"   {name} ({res['seconds']:.2f}s): " + "; ".join(
                    ", ".join(f"{k}={v}" for k, v in r.items()) for r in (res["rows"] or [])[:3]
                    if name != "hazard" or r["age_d"] in HAZARD_ROWS))
        print("\npooled daily close hazard (at risk = open at age a and observable through a+1 d):")
        print(" age  at-risk  done  drop  h_done  h_drop")
        for h in pooled["hazard"]:
            if h["age_d"] in HAZARD_ROWS:
                print(f" {h['age_d']:3d}  {h['at_risk']:6d}  {h['done_in']:4d}  {h['dropped_in']:4d}  {h['h_done']:.3f}  {h['h_drop']:.3f}")
        b = pooled["backlog_days"]
        print(f"pooled backlog days: widened {b['widened']} · narrowed {b['narrowed']} · flat {b['flat']} of {b['days']}")
    if "--check" in sys.argv:
        published = os.path.join(_HERE, "FORMAT-views.sql")
        try:
            with open(published, encoding="utf-8") as f:
                drift = f.read() != sql_text()
        except OSError:
            drift = True
        if not out or failures or drift:
            print(f"check: FAIL — {failures or ('FORMAT-views.sql drifted from --sql' if drift else 'no stores')}")
            return 1
        print(f"check: ok — {sum(len(p) for p in out.values())} probes, 0 errors; FORMAT-views.sql current")
    return 0


if __name__ == "__main__":
    sys.exit(main())
