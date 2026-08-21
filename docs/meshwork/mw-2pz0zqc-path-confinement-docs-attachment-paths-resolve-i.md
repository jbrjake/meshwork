---
id: mw-2pz0zqc
title: "Path confinement: docs:/attachment paths resolve inside the repo only"
status: open
category: core/verify
discovered-from: mw-mjwfvxn
verify: run cargo test e2e::path_confinement
docs:
  - DESIGN-meshwork.md#§-12b-trust-boundary
created: 2026-08-07T01:55Z
seq: 160
handoff: |
  Next after the 2026-08-20 session (verify-security umbrella closed;
  changed-verify surfacing, cache-key pin, schema pin, stray-tail repair,
  doing-rot pressure all landed). This is the §12b-adjacent surface:
  docs:
  and attachment paths are attacker-supplied strings the tool joins onto
  the repo root at read time. Prior art to lift, not reinvent:
  verify_exec::safe_join (src/verify_exec.rs) already refuses absolute
  paths and any `..` segment — same rule, share it (a model-tier home
  like
  docs.rs or a small path module; verify_exec can re-export). Read sites:
  docs::resolve (show --docs excerpts + lint doc checks), attachment joins
  in lint::check_budgets and the attach/show paths in src/cli/notes.rs and
  show.rs. Write site: attach records the relative path. Decide symlinks
  explicitly: lexical checks pass symlink escapes; canonicalize fails on
  not-yet-existing targets — the safe shape is lexical refusal plus a
  canonicalize-when-exists comparison, refusing on mismatch. Test
  e2e::path_confinement is red-first: an absolute docs: link, a ../
  attachment, and (on unix) a symlink out of the repo must all refuse
  loudly and run nothing.
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
