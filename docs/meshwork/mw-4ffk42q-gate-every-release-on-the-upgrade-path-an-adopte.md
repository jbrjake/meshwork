---
id: mw-4ffk42q
title: Gate every release on the upgrade path — an adopter as earlier releases left it must come out current and run every verb
category: plugin/upgrade
seq: 150
needs: [mw-x5yn4rg, mw-26j4tq5]
docs: [docs/DESIGN-meshwork.md#13-test-architecture--fixture-corpus-mw-j4j6, docs/DESIGN-meshwork.md#14-gate--verify_meshworksh-mw-j5]
verify: run cargo test package=meshwork target=suite e2e::adopter_upgrade_path
status: open
created: 2026-09-28T13:54Z
---
Releases shipped for months while every adopter's upgrade silently left
the shim behind, and no test noticed, because nothing tests a repo moving
from one release to the next. Every test builds its fixture fresh from the
current tree.

`e2e::adopter_upgrade_path`: for each adopter footprint an earlier release
produced (each shim text install.md has ever carried, recovered from git
history; the legacy root `./meshwork`; a missing shim; a per-repo
SessionStart hook; a pin one or more releases back), run the plugin's
SessionStart hook with the stub `gh`, then assert:

- the shim is byte-identical to canonical and the pin is the plugin's;
- every verb in `meshwork --help` runs through the resulting shim without
  an approval-shaped or not-found failure;
- a session carrying only `CLAUDE_CODE_SESSION_ID` resolves its session
  author through the shim;
- prime shows no stale-footprint line afterward.

Build the footprint list from git history of install.md and adopt.md, not
from memory, so a shim change that forgets to extend it fails here. It runs
in `./verify_meshwork.sh` like the rest of the suite; confirm cut-release.sh
cannot tag without a green gate. Watch it fail on a deliberate break (make
the hook skip the shim rewrite) before trusting it.

## log
- 2026-09-28T13:54Z created
