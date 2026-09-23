# meshwork v0.5.1

Some fixes from applied usage of 0.5.0. Three things that got in the way of ordinary work now behave.

## a check that names the file it will create

`lint` warns when a task's `verify:` reads a file that is not in the tree, because such a task can never close and looks like unfinished work. On the first store to run that warning, every hit was... a file the task exists to write.

An `exists` arm now declares its path a deliverable. Inside `all(…)`, a `contains` or `lacks` arm on that same path gets ignored until the file appears. An `exists` with a `*` covers every name it would match.

```
verify: all(exists docs/benchmarks-clickbench.md, contains docs/benchmarks-clickbench.md /Q0 .*ok/)
```

On the other hand, a `contains` on a path no `exists` arm names still warns, just like before.

## `lint --explain <code>` shows one code

`--explain` used to unfolded a folded finding, or put a note above a heuristic one. In either case it printed the whole report. But for any other code, it always printed the whole report, so `lint --explain needs-behind | head` showed twenty rows of everything else.

It now only shows one code's report. The note comes first when the code has one, then that code's rows alone, then a total count.

```
$ meshwork lint --explain needs-behind
no needs-behind findings; codes in this report: handoff-stale, verify-shell
0 needs-behind findings — 0 error(s), 12 warning(s) in all
```

Note that `--json` is unchanged and still carries every finding.

## `add --batch` takes plain values like the flags do

A batch document is YAML, and YAML reads a `: ` inside a plain value as a nested mapping. A title such as `marasi: consume the bytes value`, or a `verify:` quoting `"test result: ok"`, refused the whole batch with a line and column and nothing written. `add "title" --verify …` never had the problem, because it quotes what YAML would misread before writing the file.

The batch reader now does that too. A title, category, `verify:`, or other free-text value is taken as written, quoted or not. A single value in a list slot (`needs: @sibling`, `labels: ask`) becomes a one-element list, as `docs:` already did. A value already quoted stays as written.

## getting it

darwin arm64, linux arm64/x86_64, windows x86_64. Pin: put `v0.5.1` in `.meshwork-version`; install to `~/.meshwork/versions/v0.5.1/` (see the meshwork adoption skill).
