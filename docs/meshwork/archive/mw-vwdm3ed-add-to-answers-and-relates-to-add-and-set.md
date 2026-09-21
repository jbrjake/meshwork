---
id: mw-vwdm3ed
title: Add --to, --answers and --relates to add and set
category: core/authoring
needs: [mw-vffwacx, mw-tkgvsdz]
seq: 480
verify: run cargo test e2e::add_set_frontmatter_flags
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-2-3-the-fields
  - docs/ASKS-analytics-and-field-study.md#§-4-what-has-a-downstream
status: done
created: 2026-09-07T16:32Z
---
The two fields the cross-repo mechanism depends on have no CLI path; twelve of twelve writes in
the leras ask sessions went through `Write`, `perl -0pi` or a heredoc, one broke lint, one voided
a verify approval. With R-B item B.1: `--to <repo>` (scalar), `--answers <gid>` (scalar),
`--relates <gid>` (repeatable) on `add` and `set`, targets validated the way `d-add-validates`
does; `set` replaces `to:`/`answers:` and appends `relates:`. Retire the frontmatter-only error
text for these three; re-bless `e2e::cli_surface_frozen` with a reviewed diff.

## log
- 2026-09-07T16:32Z created
- 2026-09-21T16:16Z open→doing — claimed by claude (b80d763e-77c7-4aa5-8ead-40c1daaa1288)
- 2026-09-21T16:28Z doing→done — verify exit 0 @ bb021ac+3
