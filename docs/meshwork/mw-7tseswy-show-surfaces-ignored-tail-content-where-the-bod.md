---
id: mw-7tseswy
title: "show surfaces ignored tail content where the body would have been"
category: core/render
needs: [mw-n3xgfs0]
verify: run cargo test e2e::show_flags_ignored_tail_content
docs:
  - DESIGN-meshwork.md#§-6-cli-surface
  - FORMAT.md#tail-section-grammars
status: open
created: 2026-08-09T23:35Z
---
Field evidence (sazed, 2026-08-09). `show` prints schema warnings to
stderr; across two sessions they scrolled past as tool noise while the
bodies they explained were missing from the output. The reader looks at
the body area, not stderr — so put the marker there, in-body, where the
ignored content would have rendered:

    ⚠ 14 lines after ## log ignored — body belongs above the tail
      sections; lint --fix relocates them

Name the remedy, not just the fact. Needs the --fix relocation to
exist first so the message can point at it.
