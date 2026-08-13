#!/usr/bin/env python3
"""Busywork accounting, flat and compounded, from Claude Code transcripts.

Feeds the README "the numbers" tables. Two measures per session:

  flat busywork       every tracker-related byte the session put into
                      context, counted ONCE (chars/4 tokens).
  compounded busywork the same content weighted by how long it rides:
                      every API request re-submits the whole accrued
                      context, so content landing at request t of a
                      T-request session is re-paid on each of the
                      remaining T-t requests. Early content compounds
                      hard; late content barely. Per request this adds
                      min(resident busywork, context submitted), where
                      context = cache_read + cache_creation + input from
                      that request's usage record. Caching changes the
                      price of a re-submitted token, not its presence.

Busywork capture (the rule errs against meshwork: its entire CLI
surface counts):
  - tool calls on the tracker — harness todo tools (TaskCreate/
    TaskUpdate/TaskList/TaskGet/TodoWrite) or command/file_path naming
    TODO.md / HANDOFF* / check-todo / meshwork: whole input + whole
    result;
  - tracker content embedded in other calls' traffic: per-file
    `diff --git` sections whose header names a tracker file count
    whole; otherwise matching lines count (inputs and outputs);
  - the agent's own deliberation: assistant text and thinking
    paragraphs mentioning the tracker;
  - user-message paragraphs mentioning the tracker;
  - SessionStart hook injections mentioning the tracker (todo hooks,
    `meshwork prime`) count whole; other attachment/system content at
    paragraph granularity;
  - subagent transcripts (<session>/subagents/*.jsonl), same rules,
    each compounding within its own context chain.

Usage:
  python3 scripts/busywork-tokens.py [--samples] <transcript.jsonl>...
  python3 scripts/busywork-tokens.py --selftest

Transcripts live in ~/.claude/projects/<encoded-repo-path>/*.jsonl.
Compaction is not modeled (none of the measured sessions compact); the
min() cap bounds the error if one does.

Session sets behind the README tables (recorded on mw-z2kmhae): each
repo's last 10 completed pre-migration sessions vs all its completed
post-migration working sessions. Excluded: the migration session itself
(one-time, mostly busywork by construction), sub-100KB transcripts,
sessions still appending when measured, and Project B's two
verify-hygiene sweeps — one-time repairs of verifies the import brought
over rotted (54% and 56% busywork by construction), which is why its
"after" column averages 3 of 5 sessions.
"""
import glob
import json
import re
import sys

PAT = re.compile(r"todo\.md|handoff|check-todo|meshwork", re.I)
ADMIN_TOOLS = {"TaskCreate", "TaskUpdate", "TaskList", "TaskGet", "TodoWrite"}
DIFF_SPLIT = re.compile(r"(?=^diff --git )", re.M)
PARA_SPLIT = re.compile(r"\n\s*\n")
HEREDOC = re.compile(r"<<-?\s*'?(\w+)'?[^\n]*\n.*?(?:\n\1\b|\Z)", re.S)
PATHISH = re.compile(r"(?:[Pp]ath|open)\(\s*[\"']([^\"']+)[\"']")


def call_target(inp):
    """What a tool call operates ON: command with heredoc bodies stripped
    (a product-code edit whose comment cites TODO.md is not a tracker
    call), plus file_path and any path-shaped strings inside the body
    (a heredoc that writes Path("TODO.md") is one)."""
    cmd = str(inp.get("command", ""))
    parts = [HEREDOC.sub(" ", cmd), str(inp.get("file_path", ""))]
    parts += PATHISH.findall(cmd)
    return " ".join(parts)


def matched_chars(text, prose):
    if prose:
        return sum(len(p) for p in PARA_SPLIT.split(text) if PAT.search(p))
    return sum(len(l) + 1 for l in text.split("\n") if PAT.search(l))


def busy_chars(text, prose=False):
    """Chars of tracker-related content inside mixed text."""
    if not text or not PAT.search(text):
        return 0
    if "diff --git" in text:
        n = 0
        for sec in DIFF_SPLIT.split(text):
            if sec.startswith("diff --git") and PAT.search(sec.split("\n", 1)[0]):
                n += len(sec)
            elif PAT.search(sec):
                n += matched_chars(sec, prose)
        return n
    return matched_chars(text, prose)


def chunks(content):
    if isinstance(content, str):
        return [content]
    if isinstance(content, list):
        return [b.get("text", "") or "" for b in content if isinstance(b, dict)]
    return []


class Session:
    def __init__(self):
        self.busy = self.total = 0  # flat chars
        self.turns = 0
        self.turned = self.replay = 0  # tokens
        self.seen = set()
        self.cls = {}  # tool_use_id -> counts-whole
        self.resident = {}  # chain -> resident busywork chars
        self.samples = []

    def _add(self, chain, busy, total, kind, frag=""):
        self.busy += busy
        self.total += total
        if busy:
            self.resident[chain] = self.resident.get(chain, 0) + busy
            self.samples.append((busy, kind, frag.replace("\n", "\\n")[:90]))

    def scan(self, path, chain):
        for line in open(path):
            try:
                e = json.loads(line)
            except json.JSONDecodeError:
                continue
            t = e.get("type")
            if t == "attachment":
                a = e.get("attachment") or {}
                c = a.get("content")
                if not isinstance(c, str) or not c:
                    continue
                hook = a.get("type") == "hook_success" and "SessionStart" in (a.get("hookName") or "")
                if hook and PAT.search(c):
                    self._add(chain, len(c), len(c), "hook", c)
                else:
                    # listings/reminders, not prose: line granularity
                    self._add(chain, busy_chars(c), len(c), "attach", c)
                continue
            if t == "system":
                c = e.get("content")
                if isinstance(c, str) and c:
                    self._add(chain, busy_chars(c), len(c), "system", c)
                continue
            if t not in ("user", "assistant"):
                continue
            msg = e.get("message") or {}
            if t == "assistant":
                u, mid = msg.get("usage"), msg.get("id")
                if u and mid and mid not in self.seen:
                    self.seen.add(mid)
                    self.turns += 1
                    ctx = (
                        (u.get("cache_read_input_tokens") or 0)
                        + (u.get("cache_creation_input_tokens") or 0)
                        + (u.get("input_tokens") or 0)
                    )
                    self.turned += ctx
                    self.replay += min(self.resident.get(chain, 0) // 4, ctx)
            content = msg.get("content")
            if isinstance(content, str):
                self._add(chain, busy_chars(content, prose=True), len(content), "user", content)
                continue
            if not isinstance(content, list):
                continue
            for b in content:
                if not isinstance(b, dict):
                    continue
                bt = b.get("type")
                if bt == "tool_use":
                    inp = b.get("input") or {}
                    target = call_target(inp)
                    admin = b.get("name") in ADMIN_TOOLS or bool(PAT.search(target))
                    self.cls[b.get("id")] = admin
                    raw = json.dumps(inp, ensure_ascii=False)
                    if admin:
                        self._add(chain, len(raw), len(raw), "call", target or str(b.get("name")))
                    else:
                        embedded = sum(busy_chars(v) for v in inp.values() if isinstance(v, str))
                        self._add(chain, embedded, len(raw), "call-embed", raw)
                elif bt == "tool_result":
                    whole = self.cls.get(b.get("tool_use_id"), False)
                    for ch in chunks(b.get("content")):
                        if whole:
                            self._add(chain, len(ch), len(ch), "result", ch)
                        else:
                            self._add(chain, busy_chars(ch), len(ch), "result-embed", ch)
                elif bt in ("text", "thinking"):
                    txt = b.get("text") if bt == "text" else b.get("thinking")
                    if isinstance(txt, str) and txt:
                        self._add(chain, busy_chars(txt, prose=True), len(txt), bt, txt)


def analyze(path):
    s = Session()
    s.scan(path, "main")
    for af in sorted(glob.glob(path[:-len(".jsonl")] + "/subagents/*.jsonl")):
        s.scan(af, af)
    return s


def fmt(n):
    return f"{n/1e6:.2f}M" if n >= 1e6 else f"{n//1000}K" if n >= 1000 else str(n)


def report(files, show_samples):
    hdr = f"{'session':<14}{'turns':>6}{'busy_tok':>10}{'work_tok':>10}{'busy%':>7}{'comp_busy':>11}{'turned':>9}{'comp%':>7}"
    print(hdr)
    tot = dict(busy=0, total=0, turns=0, turned=0, replay=0)
    for f in files:
        s = analyze(f)
        for k in tot:
            tot[k] += getattr(s, k)
        b, w = s.busy // 4, (s.total - s.busy) // 4
        pct = 100 * s.busy / s.total if s.total else 0
        cpct = 100 * s.replay / s.turned if s.turned else 0
        print(
            f"{f.rsplit('/', 1)[-1][:12]:<14}{s.turns:>6}{b:>10}{w:>10}{pct:>6.1f}%"
            f"{fmt(s.replay):>11}{fmt(s.turned):>9}{cpct:>6.1f}%"
        )
        if show_samples:
            for busy, kind, frag in sorted(s.samples, reverse=True)[:40]:
                print(f"    {busy:>7} [{kind}] {frag}")
    if len(files) > 1:
        n = len(files)
        pct = 100 * tot["busy"] / tot["total"] if tot["total"] else 0
        cpct = 100 * tot["replay"] / tot["turned"] if tot["turned"] else 0
        print(
            f"{'AVG/session':<14}{tot['turns'] // n:>6}{tot['busy'] // 4 // n:>10}"
            f"{(tot['total'] - tot['busy']) // 4 // n:>10}{pct:>6.1f}%"
            f"{fmt(tot['replay'] // n):>11}{fmt(tot['turned'] // n):>9}{cpct:>6.1f}%"
        )


HOOK = "meshwork — 3 open"
USER = "Fix the parser.\n\nAlso update TODO.md when done."
USER_BUSY = "Also update TODO.md when done."
INP1 = {"command": "cat TODO.md"}
RES1 = "- [ ] a\n- [x] b"
THINK = "Update the handoff next.\n\nThen refactor."
THINK_BUSY = "Update the handoff next."
INP2 = {"file_path": "src/main.rs"}
RES2 = "fn main() {}\nlet x = 1;"
INP3 = {"command": "git show HEAD"}
DIFF_BUSY = "diff --git a/TODO.md b/TODO.md\n+done\n"
DIFF = DIFF_BUSY + "diff --git a/src/x.rs b/src/x.rs\n+code"
AGENT_PROMPT = "Search for the handoff doc"
AGENT_TEXT = "No tracker here."


def _usage(r, c, i, o):
    return {
        "cache_read_input_tokens": r,
        "cache_creation_input_tokens": c,
        "input_tokens": i,
        "output_tokens": o,
    }


def selftest():
    import os
    import tempfile

    assert busy_chars(DIFF) == len(DIFF_BUSY), "tracker diff section counts whole"
    assert busy_chars(THINK, prose=True) == len(THINK_BUSY), "deliberation paragraph"
    assert busy_chars(RES2) == 0, "clean work output"
    assert busy_chars("x\nfoo handoff bar\ny") == len("foo handoff bar") + 1, "line mode"

    code_edit = {"command": "python3 - <<'PY'\nimport pathlib\np = pathlib.Path(\"src/lane.rs\")\n"
                            "p.write_text('/// tracked in TODO.md')\nPY"}
    assert not PAT.search(call_target(code_edit)), "comment citing the tracker is not a tracker call"
    todo_edit = {"command": "python3 - <<'PY'\nimport pathlib\np = pathlib.Path(\"docs/HANDOFF.md\")\n"
                            "p.write_text('x')\nPY"}
    assert PAT.search(call_target(todo_edit)), "heredoc writing the tracker is one"
    assert PAT.search(call_target({"command": "wc -l TODO.md && git commit -F - <<'EOF'\nmsg\nEOF"})), \
        "tracker named outside the body"

    main_lines = [
        {"type": "attachment", "attachment": {"type": "hook_success", "hookName": "SessionStart:startup", "content": HOOK}},
        {"type": "user", "message": {"content": USER}},
        {"type": "assistant", "message": {"id": "m1", "usage": _usage(0, 100, 5, 10), "content": [
            {"type": "tool_use", "id": "t1", "name": "Bash", "input": INP1}]}},
        {"type": "user", "message": {"content": [{"type": "tool_result", "tool_use_id": "t1", "content": RES1}]}},
        {"type": "assistant", "message": {"id": "m2", "usage": _usage(200, 0, 5, 10), "content": [
            {"type": "thinking", "thinking": THINK},
            {"type": "tool_use", "id": "t2", "name": "Read", "input": INP2}]}},
        {"type": "user", "message": {"content": [{"type": "tool_result", "tool_use_id": "t2", "content": RES2}]}},
        {"type": "assistant", "message": {"id": "m3", "usage": _usage(300, 0, 10, 5), "content": [
            {"type": "tool_use", "id": "t3", "name": "Bash", "input": INP3}]}},
        {"type": "user", "message": {"content": [{"type": "tool_result", "tool_use_id": "t3", "content": DIFF}]}},
        {"type": "assistant", "message": {"id": "m4", "usage": _usage(400, 0, 1, 1), "content": [
            {"type": "text", "text": "Done."}]}},
    ]
    agent_lines = [
        {"type": "user", "isSidechain": True, "message": {"content": AGENT_PROMPT}},
        {"type": "assistant", "isSidechain": True, "message": {"id": "a1", "usage": _usage(0, 0, 50, 5), "content": [
            {"type": "text", "text": AGENT_TEXT}]}},
        {"type": "assistant", "isSidechain": True, "message": {"id": "a1", "usage": _usage(0, 0, 50, 5), "content": [
            {"type": "text", "text": "dup line"}]}},
    ]
    with tempfile.TemporaryDirectory() as d:
        path = os.path.join(d, "sess.jsonl")
        with open(path, "w") as f:
            f.writelines(json.dumps(e) + "\n" for e in main_lines)
        os.makedirs(os.path.join(d, "sess", "subagents"))
        with open(os.path.join(d, "sess", "subagents", "agent-1.jsonl"), "w") as f:
            f.writelines(json.dumps(e) + "\n" for e in agent_lines)
        s = analyze(path)

    r1 = len(HOOK) + len(USER_BUSY)
    r2 = r1 + len(json.dumps(INP1)) + len(RES1)
    r3 = r2 + len(THINK_BUSY)
    r4 = r3 + len(DIFF_BUSY)
    assert s.turns == 5, s.turns
    assert s.turned == 105 + 205 + 310 + 401 + 50, s.turned
    want_busy = r4 + len(AGENT_PROMPT)
    assert s.busy == want_busy, (s.busy, want_busy)
    want_total = (
        len(HOOK) + len(USER) + len(json.dumps(INP1)) + len(RES1) + len(THINK)
        + len(json.dumps(INP2)) + len(RES2) + len(json.dumps(INP3)) + len(DIFF)
        + len("Done.") + len(AGENT_PROMPT) + len(AGENT_TEXT) + len("dup line")
    )
    assert s.total == want_total, (s.total, want_total)
    # compounding: each request re-pays what is resident in ITS chain
    want_replay = (
        min(r1 // 4, 105) + min(r2 // 4, 205) + min(r3 // 4, 310)
        + min(r4 // 4, 401) + min(len(AGENT_PROMPT) // 4, 50)
    )
    assert s.replay == want_replay, (s.replay, want_replay)


def main():
    argv = sys.argv[1:]
    if "--selftest" in argv:
        selftest()
        print("selftest ok")
        return
    files = [a for a in argv if not a.startswith("--")]
    if not files:
        sys.exit(__doc__)
    report(files, "--samples" in argv)


if __name__ == "__main__":
    main()
