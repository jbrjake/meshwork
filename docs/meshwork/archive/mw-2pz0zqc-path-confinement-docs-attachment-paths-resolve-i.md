---
id: mw-2pz0zqc
title: "Path confinement: docs:/attachment paths resolve inside the repo only"
status: done
category: core/verify
discovered-from: mw-mjwfvxn
verify: run cargo test e2e::path_confinement
docs:
  - docs/DESIGN-meshwork.md#§-12b-trust-boundary
created: 2026-08-07T01:55Z
seq: 160
---
Named as an adjacent surface in DESIGN §12b (not covered by the MW-E5
ruling — different class): `docs:` links and attachment paths are
repo-relative strings from untrusted task files. `show --docs` (M4)
reads them, `attach` writes under attachments/. Confine resolution to
the repo root: reject absolute paths, `..` traversal, and symlink
escapes before any read/write — MW-A3's "never writes outside the
repo" made mechanical. Lint warns on offending paths; e2e proves a
hostile fixture cannot read or write outside the tempdir repo.

## log
- 2026-08-07T01:55Z created
- 2026-08-22T01:08Z open→doing — claimed by claude (session_016iEafFdzwyKAtsU3AEMhaU)
- 2026-08-22T01:15Z doing→done — verify exit 0 @ d78ac8f+2
