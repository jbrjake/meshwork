---
id: mw-nd480zh
title: Grant every meshwork verb in the plugin's skill frontmatter — no meshwork verb may stop an agent for approval
category: skill
seq: 100
docs: [.claude/skills/meshwork/SKILL.md#meshwork, CLAUDE.md#hard-boundaries]
verify: "all(contains .claude/skills/meshwork/SKILL.md /^allowed-tools:/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork init\\b/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork add\\b/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork set\\b/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork show\\b/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork comment\\b/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork attach\\b/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork start\\b/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork block\\b/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork drop\\b/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork reopen\\b/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork close\\b/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork verify\\b/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork dep\\b/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork cover\\b/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork spec\\b/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork ready\\b/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork blocked\\b/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork tree\\b/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork why\\b/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork q\\b/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork search\\b/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork stats\\b/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork asks\\b/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork prime\\b/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork lint\\b/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork portfolio\\b/, contains .claude/skills/meshwork/SKILL.md /Bash.docs.meshwork.meshwork import\\b/)"
status: blocked
created: 2026-09-28T13:45Z
claimed-by: claude (05b075de-ed31-494c-b04a-af9f0dacd709)
blocked-reason: "Owner ruling needed on the grant mechanism: on Claude Code 2.1.283 the skill-frontmatter grant pre-approved nothing in 12 headless runs (the docs' own git-commit example included) and made the skill invocation itself ask; candidates are a plugin PreToolUse hook (needs explicit go) or a settings rule taught as an adoption step. Unblocks on the ruling in a session transcript."
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
- 2026-09-30T13:13Z open→doing — claimed by claude (05b075de-ed31-494c-b04a-af9f0dacd709)
- 2026-09-30T13:36Z doing→blocked — Owner ruling needed on the grant mechanism: on Claude Code 2.1.283 the skill-frontmatter grant pre-approved nothing in 12 headless runs (the docs' own git-commit example included) and made the skill invocation itself ask; candidates are a plugin PreToolUse hook (needs explicit go) or a settings rule taught as an adoption step. Unblocks on the ruling in a session transcript.

## comments
- 2026-09-30T13:36Z [claude (05b075de-ed31-494c-b04a-af9f0dacd709)] Finding 2026-09-30: the skill-frontmatter grant does not deliver the task's promise on Claude Code 2.1.283, and shipping it would add a prompt. Work reverted from the tree; the task waits on a ruling.
  
  What was built and verified before reverting: `allowed-tools` as a YAML list in SKILL.md, two entries per verb (`Bash(docs/meshwork/meshwork <verb> *)` and the `./` form), 27 verbs, `portfolio` with its trailing wildcard. The task's verify passed (exit 0, dry run), `arch::skill_names_every_verb_and_flag` passed, and smoke passed once its skill budget measured the body instead of the whole file (frontmatter never enters context; the 54 entries put the file at 10397 B against the 8192 B cap, body 7787 B).
  
  What the docs say (code.claude.com/docs/en/skills.md, read raw):
  - "Tools Claude can use without asking permission during the turn that invokes this skill. The grant clears when you send your next message." Re-invoking the skill re-applies it for that turn only. "To pre-approve tools for the whole session rather than a single turn, add allow rules to those permission settings instead."
  - "Claude Code applies a project skill's `allowed-tools` whenever you or Claude invoke the skill, including in a `-p` run in a folder you've never trusted."
  - plugins/manifest-reference.md: no plugin-level permission field; only `commands.<name>.allowedTools`. hooks.md: a PreToolUse hook may return `permissionDecision: "allow"`; plugin hooks (`hooks` in plugin.json or hooks/hooks.json) load at session start; skill-frontmatter hooks register on invocation and persist for the session.
  
  Live runs, all headless (`claude -p`, `--setting-sources project`, project settings carrying no meshwork rule, scratch clone of this repo at 3aecd56 with the built binary copied in, plugin loaded from the clone with `--plugin-dir`, model sonnet). Verbs sampled: asks, ready, verify <id>, spec list <doc>, stats --window 7d, ./…/meshwork show <id>, q "SELECT …".
  1. No grant, plugin skill invoked: skill invocation allowed; all 7 verbs denied.
  2. Grant present, plugin skill: the skill invocation itself was denied — "Execute skill: meshwork:meshwork" now needs approval. A skill that carries `allowed-tools` asks on invocation (the bundle's Skill-tool permission check auto-allows only skills that grant nothing, else asks and offers `Skill(<name>)` rules). Regression for every adopter: one new prompt per session, and nothing bought.
  3. Grant present, skill invocation pre-approved with `--allowedTools "Skill(meshwork:meshwork)"`: skill loaded, all 7 verbs still denied. Same result for the project-scope copy (`meshwork`), for a one-line string form, and run from this trusted repo instead of the clone.
  4. Rule shapes, each in its own probe skill, each denied: `docs/meshwork/meshwork *`, `target/debug/meshwork ready *`, `/bin/ls *`, `${CLAUDE_SKILL_DIR}/run.sh *`, `${CLAUDE_PROJECT_DIR}/docs/meshwork/meshwork asks *`, `mkdir *`, and the docs' own example `git commit *` (command `git commit --allow-empty -m probe`). `git status` and `ls docs` ran, but they also ran in a session where the skill invocation was denied: the built-in read-only allowlist, not the grant.
  5. Under the host permission protocol (`--input-format stream-json --permission-prompts host`) the shim's denial reads "This command requires approval"; `mkdir` reads "Claude Code asks before a shell command creates, changes or removes files there".
  6. Control: the same rule text in project settings was ignored in the clone only because the folder is untrusted (stderr: "Ignoring 1 permissions.allow entry from .claude/settings.json: this workspace has not been trusted"); the owner's user-settings rules of that text work daily.
  
  Unverified: whether an interactive session honors the grant where headless did not. One check settles it: in the scratch clone (path in this session's scratchpad, or any clone with a probe skill whose frontmatter is `allowed-tools: Bash(git commit *)`), open `claude`, invoke the probe skill, ask for `git commit --allow-empty -m probe`, and see whether it prompts.
  
  Options for the ruling:
  - A plugin PreToolUse hook shipped in the plugin (`hooks` in plugin.json), `if: "Bash(docs/meshwork/meshwork *)"` plus the `./` form, whose script allows only a single unchained `docs/meshwork/meshwork <verb> …` and lets everything else fall through. Session-wide, applies before any skill loads, at any install scope, the mechanism mw-x5yn4rg's SessionStart hook already rides. This session's auto-mode classifier refused to let me write an auto-allowing hook script, so it needs the owner's explicit go.
  - install.md and adopt.md teach the settings rule as an adoption step (`Bash(docs/meshwork/meshwork *)` and `Bash(./docs/meshwork/meshwork *)` in user or project settings), which is what works today on the owner's machine. Manual, per machine or project; the plugin cannot ship it.
  - Keep the frontmatter grant for its per-turn effect. Rejected on the evidence above: no effect observed, one prompt added.
  
  Not filed: the release cut (nothing to ship) and the follow-up that mw-gh067xt describes (its frontmatter check has nothing to check until a mechanism lands).
