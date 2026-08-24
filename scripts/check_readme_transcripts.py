#!/usr/bin/env python3
"""Replay the README's terminal transcripts against scratch stores.

Extracts fenced code blocks whose first line starts `$ `, executes each
command in document order against a scratch store in a tempdir, and
requires every pasted output line to appear in the real output, in order
(in-order subsequence: the README curates cargo noise out of its blocks,
so extra real lines are tolerated, missing or reordered pasted lines are
not). Ids are mapped pasted->minted as `add` commands run; timestamps and
git anchors are normalized on both sides. The first divergence is
reported by README line number. A line that is exactly `...` is an
elision marker, matched against nothing.

Two stores, mirroring the README's two narratives:
- fence 1 (quick-start): repo `acme`, a cargo lib whose `stuff::thing`
  test is staged red and fixed at the `...` elision — `start` red-checks
  verifies, so green-at-start staging would add a warning line the
  pasted block doesn't show.
- fences 2+: repo `demo`, id alias set to `sa` before the first add
  (init derives `de` from the dir name); the tree is committed before
  each `prime`, whose pasted digest shows a clean store line.

Each command runs under a `MESHWORK_TODAY` stamp one minute after the
last: `ready` orders by (seq, created) with no further tiebreaker, so
same-minute created stamps would leave tied rows to engine luck — the
README's row order encodes add order, and a distinct minute per command
pins exactly that.
"""

import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path

MINTED_ID = re.compile(r"\b[a-z]{2}-[a-z0-9]{7}\b")
HEREDOC = re.compile(r"<<-?\s*'?(\w+)'?")
NORM = [
    (re.compile(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}Z"), "<TS>"),
    (re.compile(r"@ [0-9a-f]{4,40}(\+[1-9][0-9]*)?"), "@ <HASH>"),
]

RED_LIB = """#[cfg(test)]
mod stuff {
    #[test]
    fn thing() {
        assert!(false, "the work is not done yet");
    }
}
"""
GREEN_LIB = """#[cfg(test)]
mod stuff {
    #[test]
    fn thing() {}
}
"""


def normalize(line):
    line = line.rstrip()
    for rx, sub in NORM:
        line = rx.sub(sub, line)
    return line


class Segment:
    def __init__(self, lineno, script, expected):
        self.lineno = lineno  # README line of the `$ ` command
        self.script = script  # command incl. any heredoc body
        self.expected = expected  # [(lineno, text)] pasted output


def transcript_fences(path):
    """All fenced blocks whose first content line starts `$ `."""
    fences, cur, in_fence = [], None, False
    for n, raw in enumerate(path.read_text().splitlines(), 1):
        if raw.lstrip().startswith("```"):
            if in_fence and cur:
                fences.append(cur)
            cur, in_fence = [], not in_fence
            continue
        if in_fence:
            cur.append((n, raw))
    return [f for f in fences if f and f[0][1].startswith("$ ")]


def split_segments(fence):
    segs, i = [], 0
    while i < len(fence):
        lineno, text = fence[i]
        script = [text[2:]]
        i += 1
        heredoc = HEREDOC.search(script[0])
        if heredoc:
            while i < len(fence):
                script.append(fence[i][1])
                i += 1
                if script[-1].strip() == heredoc.group(1):
                    break
        expected = []
        while i < len(fence) and not fence[i][1].startswith("$ "):
            expected.append(fence[i])
            i += 1
        segs.append(Segment(lineno, "\n".join(script), expected))
    return segs


class Store:
    def __init__(self, path):
        self.path = path

    def pre_command(self, script):
        pass

    def on_elision(self):
        pass


class AcmeStore(Store):
    """Quick-start scratch: cargo lib, `stuff::thing` red until the elision."""

    def on_elision(self):
        (self.path / "src" / "lib.rs").write_text(GREEN_LIB)


class DemoStore(Store):
    """Workflow scratch: commit before `prime` — its pasted store line is clean."""

    def pre_command(self, script):
        if script.startswith("meshwork prime"):
            git(self.path, "add", "-A")
            git(self.path, "commit", "-q", "-m", "stage")


def git(cwd, *args):
    subprocess.run(
        ["git", *args], cwd=cwd, check=True,
        stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True,
    )


def init_scratch_repo(tmp, name):
    repo = tmp / name
    repo.mkdir()
    git(tmp, "init", "-q", name)
    git(repo, "config", "user.name", "Replay")
    git(repo, "config", "user.email", "replay@example.invalid")
    git(repo, "config", "commit.gpgsign", "false")
    return repo


def stage_acme(tmp):
    repo = init_scratch_repo(tmp, "acme")
    (repo / "Cargo.toml").write_text(
        '[package]\nname = "acme"\nversion = "0.1.0"\nedition = "2021"\n'
    )
    (repo / "src").mkdir()
    (repo / "src" / "lib.rs").write_text(RED_LIB)
    (repo / ".gitignore").write_text("/target\n")
    git(repo, "add", "-A")
    git(repo, "commit", "-q", "-m", "scaffold")
    return AcmeStore(repo)


def stage_demo(tmp, env):
    repo = init_scratch_repo(tmp, "demo")
    subprocess.run(
        [os.environ["MESHWORK_BIN"], "init"], cwd=repo, env=env, check=True,
        stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True,
    )
    config = repo / "docs" / "meshwork" / "config.toml"
    text = config.read_text()
    assert 'alias = "de"' in text, f"init minted an unexpected alias:\n{text}"
    config.write_text(text.replace('alias = "de"', 'alias = "sa"'))
    git(repo, "add", "-A")
    git(repo, "commit", "-q", "-m", "scaffold")
    return DemoStore(repo)


def minute_clock(start="2026-08-06T21:00Z"):
    """Minting stamps one minute apart, so tied-seq rows sort by add order."""
    day, hhmm = start.split("T")
    base = int(hhmm[:2]) * 60 + int(hhmm[3:5])
    n = 0
    while True:
        h, m = divmod(base + n, 60)
        yield f"{day}T{h % 24:02}:{m:02}Z"
        n += 1


def build_env(tmp):
    env = {k: v for k, v in os.environ.items() if not k.startswith("MESHWORK_")}
    bindir = tmp / "bin"
    bindir.mkdir()
    (bindir / "meshwork").symlink_to(os.environ["MESHWORK_BIN"])
    env["PATH"] = f"{bindir}:{env.get('PATH', '')}"
    portfolio = tmp / "no-portfolio"  # empty: no registry, quiet skip
    portfolio.mkdir()
    env["MESHWORK_PORTFOLIO"] = str(portfolio)
    return env


def grow_id_map(id_map, expected, actual_text):
    """Pair pasted ids with minted ids, in order of first appearance."""
    def fresh(text, known):
        out = []
        for m in MINTED_ID.findall(text):
            if m not in known and m not in out:
                out.append(m)
        return out

    pasted = fresh("\n".join(t for _, t in expected), id_map)
    minted = fresh(actual_text, set(id_map.values()))
    id_map.update(zip(pasted, minted))


def apply_map(id_map, text):
    for pasted, minted in id_map.items():
        text = text.replace(pasted, minted)
    return text


def fail(seg, fence_line, divergent, actual_lines):
    lineno, text = divergent
    print(f"readme-transcripts: FAIL — README.md:{lineno} diverges from the real run")
    print(f"  block starts at README.md:{fence_line}")
    print(f"  command (README.md:{seg.lineno}): $ {seg.script.splitlines()[0]}")
    print(f"  pasted line never appeared (normalized): {text}")
    print("  real output (normalized):")
    for line in actual_lines:
        print(f"    {line}")
    return 1


def main():
    readme = Path(sys.argv[1] if len(sys.argv) > 1 else "README.md")
    fences = transcript_fences(readme)
    if not fences:
        print(f"readme-transcripts: FAIL — no `$ ` transcript fences in {readme}")
        return 1
    if "meshwork init" not in fences[0][0][1]:
        print("readme-transcripts: FAIL — first transcript fence is not the quick-start")
        return 1

    with tempfile.TemporaryDirectory() as tmpdir:
        tmp = Path(tmpdir)
        env = build_env(tmp)
        acme, demo = stage_acme(tmp), stage_demo(tmp, env)
        id_map, ncmds, clock = {}, 0, minute_clock()

        for idx, fence in enumerate(fences):
            store = acme if idx == 0 else demo
            for seg in split_segments(fence):
                script = apply_map(id_map, seg.script)
                store.pre_command(script)
                env["MESHWORK_TODAY"] = next(clock)
                proc = subprocess.run(
                    ["bash", "-c", script], cwd=store.path, env=env,
                    stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                    text=True, timeout=300,
                )
                ncmds += 1
                if re.search(r"\bmeshwork add\b", script):
                    grow_id_map(id_map, seg.expected, proc.stdout)
                actual = [normalize(l) for l in proc.stdout.splitlines() if l.strip()]
                cursor = 0
                for lineno, text in seg.expected:
                    if not text.strip():
                        continue
                    if text.strip() == "...":
                        store.on_elision()
                        continue
                    want = normalize(apply_map(id_map, text))
                    while cursor < len(actual) and actual[cursor] != want:
                        cursor += 1
                    if cursor == len(actual):
                        return fail(seg, fence[0][0], (lineno, text), actual)
                    cursor += 1

    print(f"readme-transcripts: OK — {len(fences)} fences, {ncmds} commands replayed clean")
    return 0


if __name__ == "__main__":
    sys.exit(main())
