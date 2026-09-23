---
id: mw-x82yqq3
title: "Let an exists arm declare its path a deliverable — verify-path-missing skips contains and lacks arms on a path an exists arm in the same verify names"
category: lint/verify
relates: [sazed#sa-evxy8rt]
seq: 100
docs:
  - docs/DESIGN-meshwork.md#§-6-cli-surface
  - src/lint_verify.rs
verify: run cargo test lint::verify_path_missing_honors_exists_arm
status: done
created: 2026-09-23T18:32Z
---
sazed's first session on v0.5.0 counted `verify-path-missing` firing on 14 live tasks, and
all 14 were deliverables — files the task exists to write, content-checked with
`contains` because a bare `exists` passes on an empty stub. Zero rot in the sample. The
ask (`sazed#sa-evxy8rt`): inside `all(…)`, an `exists <p>` arm declares `p` a deliverable,
and the rule skips `contains`/`lacks` arms on that same `p`. `sa-wzafnqg` is already
spelled that way and still fires today.

Shape: `lint_verify::missing_read_path` collects the `Exists` paths of the same classified
verify (a glob arm matches the way `verify_exec::glob_exists` matches — literal prefix and
suffix around the one `*`) and skips a `Contains`/`Lacks` path any of them names. Legacy
shell is untouched. `check_docs` already exempts a docs link naming the task's own `exists`
deliverable; this is the same rule on the field that decides closability.

Docs in the same commit: DESIGN §6 `lint` row (`verify-path-missing` clause), SKILL.md
verifies bullet (the artifact-task shape gains the content arm beside the `exists` arm).

## log
- 2026-09-23T18:32Z created
- 2026-09-23T18:32Z open→doing — claimed by claude (c86c9e3d-7fc8-4611-b368-47218aec1288)
- 2026-09-23T18:43Z doing→done — verify exit 0 @ d2f9543+4
