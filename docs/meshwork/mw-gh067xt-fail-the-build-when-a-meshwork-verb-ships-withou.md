---
id: mw-gh067xt
title: Fail the build when a meshwork verb ships without its plugin permission grant
category: skill
seq: 110
needs: [mw-nd480zh]
docs: [CLAUDE.md#hard-boundaries, docs/DESIGN-meshwork.md#13-test-architecture--fixture-corpus-mw-j4j6]
verify: "all(run cargo test package=meshwork target=suite arch::skill_grants_every_verb, contains CLAUDE.md /arch::skill_grants_every_verb/)"
status: open
created: 2026-09-28T13:45Z
handoff: |
  Owner ruling 2026-09-30, in session: no release is cut until the task
  queue is empty, and the owner calls the cut in the transcript. Do not
  file, propose or sequence a cut or release notes. Work the queue.
  
  Owner ruling 2026-09-30, in session: nothing shipped to adopters may
  rely
  on `gh` — the upgrade hook fetches with curl against the public
  release
  URL. Keep that in mind if this task's lockstep sweep touches fetch
  paths.
  
  Where things stand for this task: every verb is granted (mw-nd480zh,
  proven
  in the owner's interactive run), so the red-first order in the body is
  now:
  write the test, watch it pass on the current tree, then delete one
  verb's
  pair of `allowed-tools` lines from SKILL.md's frontmatter, watch that
  verb
  named, restore.
  
  Files and symbols:
  
  - tests/suite/arch.rs — `skill_names_every_verb_and_flag` holds the
  `--help` walker (`help_of`, `named`, `walk`) as nested fns. Lift the
  walker to module level so the grant test shares it; the naming test's
  behavior must not change (its assertion messages are cited in
  CLAUDE.md).
  - .claude/skills/meshwork/SKILL.md — frontmatter `allowed-tools:`
  lists two
  `Bash(...)` entries per verb, `docs/meshwork/meshwork <verb> *` and the
  `./` form; the body budget (smoke, 8192 bytes) measures only after the
  closing `---`, so frontmatter growth is free. Parse the frontmatter with
  the `serde_yaml` already in Cargo.toml (or a strict line parser); a
  missing or misspelled key fails, never "zero checked".
  - CLAUDE.md#hard-boundaries — the grant line names its test the way
  the
  skill line names `arch::skill_names_every_verb_and_flag`; the verify's
  `contains CLAUDE.md /arch::skill_grants_every_verb/` is that edit.
  
  Proven this session (mw-x5yn4rg, gate green at its commit): the plugin
  now carries `hooks/session-start.sh` and the canonical shim
  `hooks/meshwork`; `init` embeds the shim. For this task's lockstep
  sweep:
  `hooks/meshwork` vs the `include_str!` in src/cli/init.rs is guarded by
  construction (same bytes), and the manifest's hook command path is the
  one
  gap already found — filed as mw-s7399xj, so do not re-file it.
---
Verbs shipped for months without a permission grant because nothing checked
for one. `arch::skill_names_every_verb_and_flag` (tests/suite/arch.rs)
proves SKILL.md *teaches* every verb and never looks at what the plugin
*grants*, so a verb could land taught but still stopping the agent for
approval, and the gate stayed green. The testing process checked the skill's
prose and nothing the plugin does in Claude Code.

The work:

- `arch::skill_grants_every_verb` in tests/suite/arch.rs. Walk `--help`
  exactly as the naming test does (factor the walker out and share it; do
  not copy it), parse SKILL.md's YAML frontmatter, and fail naming every
  top-level verb with no `allowed-tools` entry in the canonical shim form.
  A missing or misspelled `allowed-tools` key, or frontmatter that does not
  parse, fails loudly and never passes vacuously as "zero grants checked".
- Red first, and watch it fail twice before trusting it: once on the
  current tree (no grants), and once on a deliberate break after the grant
  task lands (delete one verb's entry, see that verb named, restore).
- Extend the "plugin's permission grant is part of the feature" line in
  CLAUDE.md's Hard boundaries to name `arch::skill_grants_every_verb` as
  its enforcement, the way the skill line names its test.
- Check that nothing else the plugin should keep in lockstep with the
  binary is unguarded (sub-verbs under a granted wildcard, plugin.json
  version vs tag), and file a task for each gap you find instead of
  widening this one.

## log
- 2026-09-28T13:45Z created
- 2026-09-30T15:14Z handoff by claude (c25add50-6c7d-4332-bb50-42705a33eb64)
- 2026-09-30T15:14Z handoff by claude (c25add50-6c7d-4332-bb50-42705a33eb64)
