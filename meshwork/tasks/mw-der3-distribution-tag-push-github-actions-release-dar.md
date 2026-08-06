---
id: mw-der3
title: "Distribution: tag-push GitHub Actions release (darwin binary) + per-project pinned install"
status: open
category: meta/distribution
verify: test -f .github/workflows/release.yml
docs:
  - REQUIREMENTS-meshwork.md#§-j-non-functional   # MW-J3 adoptable-in-one-session
created: 2026-08-06
handoff: |
  Remote exists now (jbrjake/meshwork) — that was the blocker. Ship the
  workflow minimal: tag push → build darwin arm64 → attach tar.gz to the
  release; no matrix, no cross-compile, nobody else consumes this yet.
  The pin contract is .meshwork-version + ~/.meshwork/versions/<tag>/ —
  the adoption skill already documents it, keep the paths in sync. CI is
  the one thing the gate can't verify; watch the first run to completion.
---
Owner ruling 2026-08-06: NO global cargo install — each consuming repo chooses
its own meshwork version. Model: repo commits a version pin (e.g.
.meshwork-version with a tag); tag push builds a darwin binary via GitHub
Actions and attaches it to a Release; binaries cache per-version under
~/.meshwork/versions/<tag>/ so repos share downloads; hooks invoke the pinned
binary. Prereqs done: LICENSE (MIT), Cargo.toml license=MIT. Still needed:
GitHub remote + first tag. The adoption skill (~/.claude/skills/meshwork,
machine-local) documents the model and must be updated when this lands.
Blocks mw-ntt5 — the sazed pilot must install the pinned way, not ad hoc.

## log
- 2026-08-06 created
