---
id: mw-1cmpywn
title: "Put the four session rules in prime's footer in two lines, inside the MW-D3 budget"
category: core/render
needs: [mw-vffwacx]
seq: 560
verify: run cargo test e2e::prime_footer_rules
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-3-recommendations
status: done
created: 2026-09-07T16:32Z
---
With R-B item B.6: the plugin's SessionStart hook and this repo's `.claude/settings.json` print
the skill body after `prime` (≈ 8 KB; the prime budget is untouched). Plugin-side change carried
through the install and migrate references so adopters pick it up on the next pin.

## log
- 2026-09-07T16:32Z created
- 2026-09-21T16:36Z open→doing — claimed by claude (b80d763e-77c7-4aa5-8ead-40c1daaa1288)
- 2026-09-21T16:39Z doing→done — verify exit 0 @ 0eed1b1+3
