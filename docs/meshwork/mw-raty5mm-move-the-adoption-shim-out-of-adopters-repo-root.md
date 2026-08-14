---
id: mw-raty5mm
title: Move the adoption shim out of adopters' repo roots into docs/meshwork/
category: skill
seq: 185
verify: contains .claude/skills/meshwork/references/install.md docs/meshwork/meshwork
status: open
created: 2026-08-14T19:31Z
---
Owner ruling 2026-08-14: it is NOT OKAY for adoption to create a top-level
`./meshwork` shim in someone's repo — that pollutes their project root, and
footprint decisions like that are never made without asking. Going forward
the shim lives at `docs/meshwork/meshwork` — the store directory is already
meshwork's only sanctioned footprint. The work: install.md + adopt.md teach
the new path (shim location, hook/script invocations, PATH-free usage
examples); existing adopters (sazed, leras) migrate their root shims on
their next upgrade, called out in the upgrade notes. This repo's own root
shim is the owner's call, not part of this task. NOTE: marketplace installs
resolve the latest tag, so this reaches adopters only when the next tag is
cut — until then every adoption plants a root shim.

## log
- 2026-08-14T19:31Z created
