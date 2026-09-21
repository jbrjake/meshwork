---
id: mw-b3c7ekt
title: "Match GitHub's heading slugs in docs: anchors, and name the nearest heading when an anchor misses"
category: core/hygiene
answers: sazed#sa-rj7vxkt
seq: 360
verify: run cargo test lint::anchor_github_slug
docs:
  - docs/DESIGN-meshwork.md#§-6-cli-surface
  - docs/REQUIREMENTS-meshwork.md#§-2-requirements
status: done
created: 2026-09-21T15:28Z
---
`anchor-missing` fires on eight live sazed tasks; four of them carry anchors that are exactly
what GitHub renders for the heading. Meshwork's slug hyphenates every non-alphanumeric run,
GitHub drops punctuation without a separator: `3.2` → `32`, `engine's` → `engines`, and an
emoji or em dash between spaces leaves `--`. An anchor copied from a rendered link therefore
never matches under the meshwork rule alone.

Fix, in `src/docs.rs`: a second slug that follows GitHub's rule (lowercase; keep alphanumerics,
`-`, `_` and spaces; spaces to hyphens), applied to both sides, and an anchor resolves when
either rule prefix-matches at a `-` boundary — every anchor that matched before still does.
When neither matches, the finding names the nearest heading's slug (`nearest heading: #…`), so
a true miss shows the disagreement instead of asking the author to guess. The other four sazed
findings are true positives — renamed headings and a dropped section number — and keep firing,
now with the hint.

Ships in the next release; sazed closes its side on the pin bump.

## log
- 2026-09-21T15:28Z created
- 2026-09-21T15:29Z open→doing — claimed by claude (b80d763e-77c7-4aa5-8ead-40c1daaa1288)
- 2026-09-21T15:34Z doing→done — verify exit 0 @ 7c49b77+3
