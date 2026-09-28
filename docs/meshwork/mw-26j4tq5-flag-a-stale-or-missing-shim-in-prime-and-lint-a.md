---
id: mw-26j4tq5
title: "Flag a stale or missing shim in prime and lint, and let lint --fix rewrite it"
category: plugin/upgrade
seq: 140
needs: [mw-x5yn4rg]
docs: [.claude/skills/meshwork/SKILL.md#meshwork, CLAUDE.md#hard-boundaries]
verify: run cargo test package=meshwork target=suite e2e::shim_stale_flagged_and_fixed
status: open
created: 2026-09-28T13:54Z
---
The binary has no idea what shim it runs through. The backstop for when
the plugin hook could not run: the binary embeds the canonical shim (the
same file the plugin ships), and when `docs/meshwork/meshwork` differs from
it or is missing:

- prime leads with one line naming it and the fix;
- `lint` reports `shim-stale` / `shim-missing`, with `--explain` showing the
  diff;
- `lint --fix` rewrites the shim to canonical, executable.

This repo's own shim execs `target/debug/meshwork` on purpose. Handle that
as an explicit, tested exception, not a silent skip. SKILL.md names the
lint codes in the same commit (`arch::skill_names_every_verb_and_flag`
does not cover lint codes, so check by hand).

## log
- 2026-09-28T13:54Z created
