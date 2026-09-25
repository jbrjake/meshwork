# meshwork v0.5.2

Spit and polish on the SKILL.md. Meshwork itself is unchanged, but agents may use it more effectively.

## the skill teaches every verb

The skill now includes the "daily-use" tips in the skill itself, with rarer rituals in references. In addition, it now covers requests between repos, analytics, pinning tasks to spec doc clauses, lint codes, and everything else in the help.

The skill also now names, shames, and offers better choices versus the shortcuts agents have been observed taking in real-world usage: rewriting failing checks rather than doing the work, writing `status: done` by hand, preferring `grep -r` over `search`, and building around an open request.

## the skill shouldn't fall behind again

A new build check walks every `--help` screen and fails when a verb, sub-verb, or `add`/`set` flag is missing from the skill.

## getting it

The skill comes from the plugin, which tracks the newest tag: `/plugin install meshwork@jbrjake` on a new machine, or the plugin's update command where it is already installed.

darwin arm64, linux arm64/x86_64, windows x86_64. Pin: put `v0.5.2` in `.meshwork-version`; install to `~/.meshwork/versions/v0.5.2/` (see the meshwork adoption skill).
