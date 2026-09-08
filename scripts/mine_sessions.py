#!/usr/bin/env python3
"""Score Claude Code session transcripts for meshwork usage signal. Read-only.

    python3 scripts/mine_sessions.py                 # ranked table + per-repo totals + telemetry
    python3 scripts/mine_sessions.py --json out.json # every session's signals as one document
    python3 scripts/mine_sessions.py --top 25 --min-cmds 3
    python3 scripts/mine_sessions.py --all-projects  # include repos with no meshwork store

Input: ~/.claude/projects/<slug>/*.jsonl (override with CLAUDE_PROJECTS). A session's
human prompts come from the main chain only; tool calls are counted on every chain
(subagents included). Pair with scripts/render_session.py to read a chosen session.

Signals per session
  n_user          human prompts (string content; command wrappers / hook meta excluded)
  user_mw         prompts AFTER the first that say 'meshwork' or cite a task id (xx-1a2b3c4)
  user_task_talk  prompts after the first that talk tracker-shaped (handoff, prime, file it, asks…)
  user_mw_corr    user_mw prompts that also carry correction language (no / wrong / told you / again)
  heated          prompts with CAPS runs, !! ?? ?!, or the frustration lexicon
  heat_mw         heated prompts that themselves mention meshwork / a task id
  heat_then_mut   heated prompts followed (before the next human prompt) by a mutating verb
  mw_cmds         Bash tool_uses that invoke meshwork      mw_mut  mutating subset
  mw_help         invocations that ask --help (discoverability probes, not counted as errors)
  mw_guess        invocations of verbs that are not on the CLI surface (next, addressed, inbox…)
  mw_err          meshwork results that look like errors, bucketed in err_kinds
  mw_deny         meshwork tool_uses the user declined
  store_hand_edit Edit/Write/heredoc directly on docs/meshwork/*.md (bypassing the CLI)
  skill           Skill-tool invocations of the meshwork skill

Event stream (--events, JSONL) — the event envelope
  One record per event, in one shape shared with the store's `events` view so a session's
  stream and the store's stream sort onto one timeline: the six envelope keys are the view's
  column subset, then two the transcript alone can supply.
    repo     the repo the transcript belongs to (its project directory)
    gid      `<repo>#<id>` of the task the event names, from the id's prefix; null when none
    kind     call | hand-edit | shell-read | prompt | stop
    at       the record's timestamp, ISO 8601 — the view's `at`
    actor    the author the shim stamps for this session: `claude (session_<bridge id>)` for a
             bridge session, else `claude (<transcript uuid>)`; joins to comments.author and to
             the `claimed by` log notes where the store carries the same string
    note     the command (call, shell-read), the file path (hand-edit), the prompt text (prompt),
             or the stop reason (stop) — the view's `note`; truncated to 300 chars
    session  the transcript uuid
    detail   kind-specific payload: call → turn, verbs, ids, sidechain, help, hop, waive,
             set_verify, err, err_line, denied, out_bytes; hand-edit → turn, tool, ids,
             sidechain; shell-read → turn, sidechain; prompt → turn, heated, mentions, queued
  Filter on `kind`, never on the payload's shape. The store side of the same timeline is
  `SELECT repo, gid, kind, at, actor, note FROM events` once the view is registered.
"""
import argparse
import glob
import json
import math
import os
import re
from collections import Counter, defaultdict
from datetime import datetime

PROJECTS_DIR = os.path.expanduser(os.environ.get("CLAUDE_PROJECTS", "~/.claude/projects"))
CODE_PREFIX = "-Users-jonr-Documents-code"
MW_REPOS = {"portfolio", "sazed", "leras", "meshwork", "marasi", "tensoon", "oreseur",
            "wyndam", "marasi-applied-r-and-d", "code"}   # 'code' = sessions at the code root
PREFIX = {"mw": "meshwork", "sa": "sazed", "le": "leras", "ma": "marasi", "te": "tensoon",
          "or": "oreseur", "wy": "wyndam", "ar": "marasi-applied-r-and-d", "po": "portfolio"}

SURFACE = {"init", "add", "set", "show", "comment", "attach", "start", "block", "drop", "reopen",
           "close", "verify", "dep", "ready", "blocked", "tree", "why", "q", "search", "prime",
           "lint", "mirror", "portfolio", "import", "help"}
PORTFOLIO_SURFACE = {"ready", "next", "q", "help"}
MUTATING = {"add", "set", "comment", "attach", "start", "block", "drop", "reopen", "close", "dep",
            "init", "import", "lint"}          # lint --fix rewrites; plain lint is read-only
READ = {"show", "verify", "ready", "blocked", "tree", "why", "q", "search", "prime", "portfolio",
        "help", "mirror"}

TASK_ID = re.compile(r"\b[a-z]{2}-(?=[a-z0-9]*\d)[a-z0-9]{4,7}\b")
MW_WORD = re.compile(r"\bmeshwork\b", re.I)
TASK_TALK = re.compile(r"\b(?:handoff|hand-off|prime|file (?:it|that|them|a task|tasks|an ask|the ask)|"
                       r"close (?:it|that|them|the task|out)|task store|the store|the tracker|"
                       r"ready (?:list|queue|set)|asks?\b.*\b(?:sibling|from|to)|to: task|"
                       r"verify:|the queue|what'?s next|next up|sequenc\w+)\b", re.I)
SYSREM = re.compile(r"<system-reminder>.*?</system-reminder>", re.S)
WRAPPERS = ("<command-name>", "<local-command-caveat>", "<local-command-stdout>", "<bash-input>",
            "<bash-stdout>", "<bash-stderr>", "<task-notification>", "<user-prompt-submit-hook>")

# meshwork at command position: line start or after ; && || | ( $( ` then do time exec env
BIN = (r"(?:\./|(?:~|\$HOME|/Users/\w+)?(?:/?[\w.-]+/)*)?"
       r"(?:docs/meshwork/)?(?:target/(?:debug|release)/)?meshwork")
MW_CMD = re.compile(
    r"(?:^|[;&|(`]\s*|\b(?:then|do|time|exec|command|sh -c ['\"]|bash -c ['\"]|env(?: \w+=\S+)*)\s+)"
    r"(?:\w+=\S+\s+)*" + BIN + r"\s+"
    r"(?:(?:--as|--repo|-C)\s+(?:\"[^\"]*\"|'[^']*'|\S+)\s+|--json\s+)*"
    r"(--help|-h|--version|-V|[a-z][a-z-]*)(?:\s+([a-z][a-z-]*))?", re.M)
STORE_PATH = re.compile(r"docs/meshwork/(?:archive/)?[a-z]{2}-[a-z0-9]{4,7}-[\w.-]*\.md")
HEREDOC_STORE = re.compile(r"(?:cat|tee|sed\s+-i|perl\s+-pi|python3?\s+-|>>?)\s*[^\n]*docs/meshwork/(?:archive/)?[a-z]{2}-[a-z0-9]{4,7}-[\w.-]*\.md")

CAPS_RUN = re.compile(r"\b[A-Z][A-Z'’]{2,}\b(?:[\s,.:;!?-]+\b[A-Z][A-Z'’]{2,}\b)+")
CAPS_WORD = re.compile(r"\b[A-Z]{5,}\b")
CAPS_ALLOW = set("""README CLAUDE TODO JSON HTML HTTP HTTPS GITHUB PLAN DESIGN TRACE FORMAT HANDOFF
PROPOSAL REQUIREMENTS DATAFUSION MEMORY BASELINE SKILL LICENSE SQLITE ASCII NOTES ARCHITECTURE
REVIEW AGENTS TESTING BUILD CHANGELOG CONTRIBUTING GESTALT SHOULD FIXME OKAY QUERIES DECISIONS
MIGRATION SUMMARY""".split())
PUNCT_HEAT = re.compile(r"(?:!{2,}|\?{2,}|\?!|!\?)")
FRUSTRATION = re.compile(     # deliberately narrow: profanity and explicit exasperation only
    r"\b(?:wtf|fuck\w*|shit|dammit|damn|ugh+|argh+|jesus christ|what the hell|are you kidding|"
    r"i already (?:told|said|asked)|i (?:told|asked) you (?:to|not|exactly|last)|how many times|"
    r"what are you doing|omfg|omg)\b", re.I)
CORRECTION = re.compile(
    r"\b(?:no[,.!]|nope|wrong|incorrect|not what i|that'?s not|don'?t|do not|never|stop|undo|revert|"
    r"instead|i said|i asked|again|why did you|shouldn'?t have|not supposed to|you were supposed|"
    r"you should have|missed|forgot|ignored|didn'?t (?:read|follow|use|run))\b", re.I)

ERR_KINDS = [                       # first match wins; strings come from src/cli and clap
    ("unknown-verb", re.compile(r"unrecognized subcommand")),
    ("bad-args", re.compile(r"unexpected argument|invalid value|is required|required arguments|"
                            r"tip: to pass '--\S+' as a value|the following required")),
    ("shim-missing", re.compile(r"no such file or directory: \S*meshwork|command not found: \S*meshwork|"
                                r"meshwork: (?:command )?not found|not a meshwork repo|no meshwork store")),
    ("id-not-found", re.compile(r"\b[a-z]{2}-[a-z0-9]{4,7} not found")),
    ("close-refused", re.compile(r"\bstays (?:open|doing|blocked)\b|refusing (?:unapproved|malformed) verify|"
                                 r"has no verify: — a task without a machine check")),
    ("state-refused", re.compile(r"cannot (?:start|close|block|drop|reopen) \S+: status is")),
    ("lint-parse", re.compile(r"error\[parse\]|mapping values are not allowed|while parsing")),
    ("lint-error", re.compile(r"error\[[a-z-]+\]")),
    ("dep-refused", re.compile(r"cycle: \S+ →|would create a cycle|cannot depend on itself")),
    ("panic", re.compile(r"panicked at|RUST_BACKTRACE")),
    ("nonzero-exit", re.compile(r"(?:Exit|exit) code [1-9]")),
    ("error-line", re.compile(r"(?:^|\n)\s*error[:\[]", re.I)),
]
PRIME_NEXT = re.compile(r"next → (\S+)")
PRIME_ADDRESSED = re.compile(r"addressed to this repo \((\d+)\)")


def human_text(rec):
    if rec.get("type") != "user" or rec.get("isMeta") or rec.get("isSidechain") or "toolUseResult" in rec:
        return None
    c = (rec.get("message") or {}).get("content")
    if isinstance(c, list):
        if any(isinstance(b, dict) and b.get("type") == "tool_result" for b in c):
            return None
        c = "\n".join(b.get("text", "") for b in c if isinstance(b, dict) and b.get("type") == "text")
    if not isinstance(c, str):
        return None
    t = SYSREM.sub("", c).strip()
    if not t or t.startswith(WRAPPERS):
        return None
    return t


def is_heated(t):
    if PUNCT_HEAT.search(t) or FRUSTRATION.search(t) or CAPS_RUN.search(t):
        return True
    return any(w not in CAPS_ALLOW for w in CAPS_WORD.findall(t))


def classify_error(text):
    """(kind, matched line) of the first error signature found, else None."""
    for kind, rx in ERR_KINDS:
        m = rx.search(text)
        if m:
            line_start = text.rfind("\n", 0, m.start()) + 1
            line_end = text.find("\n", m.end())
            return kind, text[line_start:line_end if line_end != -1 else None].strip()[:200]
    return None


def result_text(content):
    if isinstance(content, list):
        content = "\n".join(b.get("text", "") for b in content if isinstance(b, dict))
    return content if isinstance(content, str) else ""


def new_stats(path):
    return {"project": os.path.basename(os.path.dirname(path)), "session": os.path.basename(path)[:-6],
            "size": os.path.getsize(path), "n_user": 0, "user_mw": 0, "user_task_talk": 0,
            "user_mw_corr": 0, "heated": 0, "heat_mw": 0, "heat_then_mut": 0, "mw_cmds": 0,
            "mw_mut": 0, "mw_help": 0, "mw_guess": 0, "mw_err": 0, "mw_deny": 0,
            "store_hand_edit": 0, "skill": 0, "n_assistant": 0, "n_tool": 0, "subagents": 0,
            "sidechain_mw": 0, "first_mw": False, "title": "", "last_prompt": "", "start": "",
            "end": "", "version": "", "verbs": Counter(), "guessed": Counter(), "err_kinds": Counter(),
            "heated_samples": [], "corr_samples": [], "err_samples": [], "user_mw_samples": [],
            "guess_samples": [], "events": [], "prime_next": "", "prime_addressed": -1,
            "prime_bytes": 0, "prime_count": 0, "bridge": ""}


def gid_of(task_id):
    """`repo#id` for a bare id, by prefix; None when the prefix is not a registered repo's."""
    repo = PREFIX.get(task_id[:2]) if task_id else None
    return f"{repo}#{task_id}" if repo else None


def emit(st, kind, ts, note, ids=(), **detail):
    """Append one envelope record (repo/actor/session are stamped when the stream is written)."""
    st["events"].append({"kind": kind, "at": ts, "gid": gid_of(ids[0]) if ids else None,
                         "note": (note or "")[:300], "detail": {"ids": list(ids)[:6], **detail}})
    return st["events"][-1]["detail"]


def score_file(path):
    st = new_stats(path)
    tool_by_id = {}
    queued = set()
    pending_heat = heat_credited = first_seen = False
    with open(path, encoding="utf-8", errors="replace") as f:
        for line in f:
            try:
                rec = json.loads(line)
            except Exception:
                continue
            typ = rec.get("type")
            ts = rec.get("timestamp")
            if ts:
                st["start"] = st["start"] or ts
                st["end"] = ts
            if typ == "ai-title":
                st["title"] = rec.get("aiTitle") or st["title"]
                continue
            if typ == "last-prompt":
                st["last_prompt"] = rec.get("lastPrompt") or st["last_prompt"]
                continue
            if typ == "bridge-session":
                st["bridge"] = st["bridge"] or str(rec.get("bridgeSessionId") or "")
                continue
            if rec.get("version") and not st["version"]:
                st["version"] = rec["version"]
            if typ == "attachment":
                att = rec.get("attachment") or {}
                if str(att.get("command", "")).endswith("meshwork prime"):
                    out = att.get("stdout") or ""
                    st["prime_count"] += 1
                    if not st["prime_next"]:
                        st["prime_bytes"] = len(out.encode("utf-8"))
                        m = PRIME_NEXT.search(out)
                        st["prime_next"] = m.group(1) if m else ""
                        m = PRIME_ADDRESSED.search(out)
                        st["prime_addressed"] = int(m.group(1)) if m else 0
                continue
            if typ == "queue-operation" and rec.get("operation") == "enqueue":
                # a prompt typed while the agent was mid-turn; it may or may not be re-logged
                # as a user record later, so count it once by text
                t = (rec.get("content") or "").strip()
                if t and t not in queued:
                    queued.add(t)
                    typ, rec = "user", {"type": "user", "message": {"content": t}, "queued": True}
            if typ == "user":
                t = human_text(rec)
                if t is not None and t in queued and rec.get("uuid"):
                    t = None          # already counted from its enqueue record
                if t is not None:
                    st["n_user"] += 1
                    mentions = bool(MW_WORD.search(t) or TASK_ID.search(t))
                    heated = is_heated(t)
                    emit(st, "prompt", ts, t, TASK_ID.findall(t), turn=st["n_user"], heated=heated,
                         mentions=mentions, queued=bool(rec.get("queued")))
                    if not first_seen:
                        first_seen, st["first_mw"] = True, mentions
                    else:
                        if mentions:
                            st["user_mw"] += 1
                            if len(st["user_mw_samples"]) < 8:
                                st["user_mw_samples"].append(t[:300].replace("\n", " "))
                            if CORRECTION.search(t):
                                st["user_mw_corr"] += 1
                                if len(st["corr_samples"]) < 6:
                                    st["corr_samples"].append(t[:300].replace("\n", " "))
                        elif TASK_TALK.search(t):
                            st["user_task_talk"] += 1
                    if heated:
                        st["heated"] += 1
                        st["heat_mw"] += 1 if mentions else 0
                        if len(st["heated_samples"]) < 8:
                            st["heated_samples"].append(t[:300].replace("\n", " "))
                    pending_heat, heat_credited = heated, False
                    continue
                content = (rec.get("message") or {}).get("content")
                if isinstance(content, list):
                    for b in content:
                        if not isinstance(b, dict) or b.get("type") != "tool_result":
                            continue
                        kind = tool_by_id.get(b.get("tool_use_id"))
                        if not kind or kind[0] != "mw":
                            continue
                        txt = result_text(b.get("content"))
                        ev = st["events"][kind[3]]["detail"] if kind[3] is not None else None
                        if rec.get("toolDenialKind") or "user doesn't want to proceed" in txt \
                                or "The user doesn't want to take this action" in txt:
                            st["mw_deny"] += 1
                            if ev:
                                ev["denied"] = True
                            continue
                        if ev:
                            ev["out_bytes"] = len(txt)
                        if kind[2]:          # a --help probe: never an error
                            continue
                        hit = classify_error(txt[:800])
                        if b.get("is_error") and not hit:
                            hit = ("error-line", next((l.strip() for l in txt.splitlines() if l.strip()), "")[:200])
                        if hit:
                            ek, eline = hit
                            st["mw_err"] += 1
                            st["err_kinds"][ek] += 1
                            if ev:
                                ev["err"] = ek
                                ev["err_line"] = eline
                            if len(st["err_samples"]) < 8:
                                st["err_samples"].append(f"[{ek}] {kind[1][:140]} => {txt[:220]}".replace("\n", " "))
                continue
            if typ == "assistant":
                if not rec.get("isSidechain"):
                    st["n_assistant"] += 1
                for b in (rec.get("message") or {}).get("content") or []:
                    if not isinstance(b, dict) or b.get("type") != "tool_use":
                        continue
                    st["n_tool"] += 1
                    name, inp, tid = b.get("name", ""), b.get("input") or {}, b.get("id")
                    if name in ("Agent", "Task"):
                        st["subagents"] += 1
                    if name == "Bash":
                        cmd = inp.get("command", "") or ""
                        hits = MW_CMD.findall(cmd)
                        is_help = False
                        ev_idx = None
                        if hits:
                            st["mw_cmds"] += 1
                            st["sidechain_mw"] += 1 if rec.get("isSidechain") else 0
                            ev_idx = len(st["events"])
                            emit(st, "call", ts, cmd, TASK_ID.findall(cmd), turn=st["n_user"],
                                 verbs=[v if v != "portfolio" else f"portfolio {s or ''}".strip() for v, s in hits],
                                 sidechain=bool(rec.get("isSidechain")), help="--help" in cmd or " -h" in cmd,
                                 hop=bool(re.search(r"cd \.\./[\w-]+", cmd)), waive="--waive" in cmd,
                                 set_verify=bool(re.search(r"\bset\b.*--verify", cmd)), err=None)
                            for verb, sub in hits:
                                if verb in ("--help", "-h") or sub in ("--help", "-h") or "--help" in cmd:
                                    is_help = True
                                    st["mw_help"] += 1
                                    continue
                                if verb in ("--version", "-V"):
                                    continue
                                label = verb if verb != "portfolio" else f"portfolio {sub or ''}".strip()
                                st["verbs"][label] += 1
                                guessed = verb not in SURFACE or (verb == "portfolio" and sub and sub not in PORTFOLIO_SURFACE)
                                if guessed:
                                    st["mw_guess"] += 1
                                    st["guessed"][label] += 1
                                    if len(st["guess_samples"]) < 6:
                                        st["guess_samples"].append(cmd[:160].replace("\n", " "))
                                if verb in MUTATING and not (verb == "lint" and "--fix" not in cmd):
                                    st["mw_mut"] += 1
                                    if pending_heat and not heat_credited:
                                        st["heat_then_mut"] += 1
                                        heat_credited = True
                        if HEREDOC_STORE.search(cmd):
                            st["store_hand_edit"] += 1
                            emit(st, "hand-edit", ts, cmd, TASK_ID.findall(cmd), turn=st["n_user"],
                                 tool="shell", sidechain=bool(rec.get("isSidechain")))
                        elif re.search(r"\b(?:grep|rg|find|ls|cat|head|sed|awk)\b[^\n|]*docs/meshwork(?!/meshwork\b)", cmd):
                            emit(st, "shell-read", ts, cmd, turn=st["n_user"], sidechain=bool(rec.get("isSidechain")))
                        tool_by_id[tid] = ("mw" if hits else "other", cmd, is_help, ev_idx)
                    elif name in ("Edit", "Write", "MultiEdit", "NotebookEdit"):
                        fp = inp.get("file_path", "") or ""
                        if STORE_PATH.search(fp):
                            st["store_hand_edit"] += 1
                            emit(st, "hand-edit", ts, fp[-120:], TASK_ID.findall(fp)[:1], turn=st["n_user"],
                                 tool=name.lower(), sidechain=bool(rec.get("isSidechain")))
                        tool_by_id[tid] = ("other", fp, False, None)
                    elif name == "Skill":
                        sk = inp.get("skill") or ""
                        if "meshwork" in sk:
                            st["skill"] += 1
                        tool_by_id[tid] = ("other", "Skill " + sk, False, None)
                    else:
                        tool_by_id[tid] = ("other", name, False, None)
    try:
        a = datetime.fromisoformat(st["start"].replace("Z", "+00:00"))
        b = datetime.fromisoformat(st["end"].replace("Z", "+00:00"))
        st["duration_min"] = int((b - a).total_seconds() // 60)
    except Exception:
        st["duration_min"] = 0
    used = 1 if st["mw_cmds"] else 0
    s = (4.0 * st["user_mw"] + 6.0 * st["user_mw_corr"] + 5.0 * st["heat_then_mut"]
         + 4.0 * st["heat_mw"] + 1.5 * min(st["heated"], 8) * used + 1.0 * min(st["user_task_talk"], 6)
         + 2.0 * math.log2(1 + st["mw_cmds"]) + 1.5 * math.log2(1 + st["mw_mut"])
         + 2.0 * min(st["mw_err"], 10) + 1.5 * min(st["mw_guess"], 6) + 5.0 * st["mw_deny"]
         + 2.5 * min(st["store_hand_edit"], 6) + 1.0 * min(st["mw_help"], 5) + 1.0 * st["skill"])
    st["score"] = round(s, 1)
    return st


def repo_of(slug):
    if slug == CODE_PREFIX:
        return "code"
    return slug.rsplit(CODE_PREFIX + "-", 1)[-1] if slug.startswith(CODE_PREFIX + "-") else slug


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--json")
    ap.add_argument("--top", type=int, default=40)
    ap.add_argument("--min-cmds", type=int, default=1)
    ap.add_argument("--all-projects", action="store_true")
    ap.add_argument("--events", help="write the session event stream as JSONL in the event envelope "
                                     "(input to mine_telemetry.py)")
    args = ap.parse_args()
    rows = []
    for d in sorted(glob.glob(os.path.join(PROJECTS_DIR, "*"))):
        repo = repo_of(os.path.basename(d))
        if not args.all_projects and repo not in MW_REPOS:
            continue
        for p in glob.glob(os.path.join(d, "*.jsonl")):
            st = score_file(p)
            st["repo"], st["path"] = repo, p
            rows.append(st)
    rows.sort(key=lambda r: -r["score"])
    if args.events:
        with open(args.events, "w") as f:
            for r in rows:
                actor = f"claude (session_{r['bridge'][4:]})" if r["bridge"].startswith("cse_") else f"claude ({r['session']})"
                for e in r["events"]:
                    f.write(json.dumps({"repo": r["repo"], "gid": e["gid"], "kind": e["kind"], "at": e["at"],
                                        "actor": actor, "note": e["note"], "session": r["session"],
                                        "detail": e["detail"]}) + "\n")
    if args.json:
        with open(args.json, "w") as f:
            json.dump([{**r, "verbs": dict(r["verbs"]), "guessed": dict(r["guessed"]),
                        "err_kinds": dict(r["err_kinds"]), "events": len(r["events"])} for r in rows], f, indent=1)
    per = defaultdict(Counter)
    for r in rows:
        c = per[r["repo"]]
        c["sessions"] += 1
        c["with_mw"] += 1 if r["mw_cmds"] else 0
        for k in ("mw_cmds", "mw_mut", "mw_help", "mw_guess", "mw_err", "user_mw", "heated", "store_hand_edit"):
            c[k] += r[k]
    print("repo\tsessions\twith_mw\tcmds\tmut\thelp\tguess\terr\tuser_mw\theated\thand_edit")
    for k, v in sorted(per.items(), key=lambda kv: -kv[1]["mw_cmds"]):
        print("\t".join(str(x) for x in [k, v["sessions"], v["with_mw"], v["mw_cmds"], v["mw_mut"], v["mw_help"],
                                         v["mw_guess"], v["mw_err"], v["user_mw"], v["heated"], v["store_hand_edit"]]))
    print()
    hdr = ["score", "repo", "session", "n_user", "u_mw", "talk", "corr", "heat", "h_mw", "h→mut", "cmds",
           "mut", "help", "guess", "err", "deny", "hand", "skill", "sub", "min", "MB", "title"]
    print("\t".join(hdr))
    for r in rows[:args.top]:
        if r["mw_cmds"] < args.min_cmds:
            continue
        print("\t".join(str(x) for x in [
            r["score"], r["repo"], r["session"][:8], r["n_user"], r["user_mw"], r["user_task_talk"],
            r["user_mw_corr"], r["heated"], r["heat_mw"], r["heat_then_mut"], r["mw_cmds"], r["mw_mut"],
            r["mw_help"], r["mw_guess"], r["mw_err"], r["mw_deny"], r["store_hand_edit"], r["skill"],
            r["subagents"], r["duration_min"], round(r["size"] / 1e6, 1),
            (r["title"] or r["last_prompt"] or "")[:64].replace("\n", " ")]))
    allv, allg, alle = Counter(), Counter(), Counter()
    for r in rows:
        allv.update(r["verbs"]); allg.update(r["guessed"]); alle.update(r["err_kinds"])
    print("\nverbs:", allv.most_common(40))
    print("guessed verbs (not on the surface):", allg.most_common(40))
    print("error kinds:", alle.most_common())


if __name__ == "__main__":
    main()
