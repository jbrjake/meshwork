---
id: mw-gh067xt
title: Fail the build when a meshwork verb ships without its plugin permission grant
category: skill
seq: 110
needs: [mw-nd480zh]
docs: [CLAUDE.md#hard-boundaries, docs/DESIGN-meshwork.md#13-test-architecture--fixture-corpus-mw-j4j6]
verify: "all(run cargo test package=meshwork target=suite arch::skill_grants_every_verb, contains CLAUDE.md /arch::skill_grants_every_verb/)"
status: done
created: 2026-09-28T13:45Z
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
- 2026-10-04T15:22Z open→doing — claimed by claude (5566559a-b6fd-4780-8c73-3214f8c33f5d)
- 2026-10-04T15:33Z doing→done — verify exit 0 @ 47f4cc4+3
