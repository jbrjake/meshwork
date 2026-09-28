---
id: mw-nd480zh
title: Grant every meshwork verb in the plugin's skill frontmatter — no meshwork verb may stop an agent for approval
category: skill
seq: 100
docs: [.claude/skills/meshwork/SKILL.md#meshwork, CLAUDE.md#hard-boundaries]
verify: "all(contains .claude/skills/meshwork/SKILL.md /^allowed-tools:/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork init\\b/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork add\\b/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork set\\b/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork show\\b/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork comment\\b/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork attach\\b/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork start\\b/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork block\\b/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork drop\\b/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork reopen\\b/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork close\\b/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork verify\\b/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork dep\\b/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork cover\\b/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork spec\\b/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork ready\\b/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork blocked\\b/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork tree\\b/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork why\\b/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork q\\b/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork search\\b/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork stats\\b/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork asks\\b/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork prime\\b/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork lint\\b/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork portfolio\\b/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork import\\b/)"
status: open
created: 2026-09-28T13:45Z
---
This repo vends the `meshwork@jbrjake` Claude Code plugin
(`.claude-plugin/plugin.json` + `.claude/skills/meshwork/`), and the plugin
is responsible for letting agents run meshwork's own verbs without an
approval prompt. It grants nothing: SKILL.md's frontmatter carries `name`
and `description` only, and `git log --all -S allowed-tools` over the skill
is empty, so every verb (the 0.5.x additions `asks`, `cover`, `spec`,
`verify`, `stats`, `search` included) stops the agent for approval unless
some settings file happens to cover it.

The work:

- Add `allowed-tools` to SKILL.md's frontmatter with one entry per verb in
  `meshwork --help` (27 today; `mirror` is not built, `help` is clap's),
  in the form the skill teaches: `docs/meshwork/meshwork <verb>`. Add the
  `./docs/meshwork/meshwork <verb>` form if the grant matches on the literal
  command prefix. `portfolio` is granted with a trailing wildcard covering
  its sub-verbs (ready, next, q, seq, stats, search, spec).
- Confirm the exact entry syntax against Claude Code's skill/plugin docs
  (`Bash(<prefix>:*)` vs `Bash(<prefix> *)`) and when the grant applies
  (skill loaded, or for the whole session). If a plugin can ship a grant
  that applies before the skill loads, use it, and write the finding into
  the task as a comment.
- Prove it live: in a fresh session in a scratch clone whose settings carry
  no meshwork rule, load the skill and call a sample of verbs, including a
  0.5.x one (`asks`, `verify <id>`, `spec list`). None may prompt. Record
  the session and the calls in a comment.
- The published plugin is the release tag, so installed copies pick up the
  grant only at the next cut (scripts/cut-release.sh, owner-gated). File the
  cut as its own task when this closes.

The verify fails today on `^allowed-tools:` and on every verb arm. The
companion check (the next task) makes the verb list self-maintaining.

## log
- 2026-09-28T13:45Z created
