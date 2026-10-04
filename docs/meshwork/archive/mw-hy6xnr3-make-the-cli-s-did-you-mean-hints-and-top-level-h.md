---
id: mw-hy6xnr3
title: Make the CLI's did-you-mean hints and top-level help match the verbs that exist
category: core/cli
docs: [docs/DESIGN-meshwork.md#6-cli-surface-complete-for-v1--anything-not-here-is-a-non-goal]
verify: "all(run cargo test forgiveness_hints_match_the_surface, contains src/cli/mod.rs /Raw SQL over .*covers/)"
status: done
created: 2026-10-02T13:39Z
---
Three messages point users wrong:
- `start x --body y` says `--body` "is set at creation … or by hand-edit in the task file", but `set` has `--body`, `--from` and `--parent` (src/cli/mod.rs flag forgiveness).
- `dep show x` says "`portfolio show` does not exist": the `show` entry fires under any parent verb.
- `meshwork --help` describes `q` as "Raw SQL over tasks/edges/labels/comments/log/repos", leaving out `covers`.

## log
- 2026-10-02T13:39Z created
- 2026-10-04T14:41Z open→doing — claimed by claude (70b8c3c2-a3ae-4cee-81aa-681b84b44405)
- 2026-10-04T14:55Z doing→done — verify exit 0 @ b6709d9+6
