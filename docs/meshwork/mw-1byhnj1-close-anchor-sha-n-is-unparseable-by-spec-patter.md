---
id: mw-1byhnj1
title: "Close anchor (@ sha[+N]) is unparseable by spec — pattern or column"
category: core/format
relates: [mw-ntn0t32]
verify: cargo test format::close_anchor_pattern
docs:
  - FORMAT.md#tail-section-grammars
status: open
created: 2026-08-09T23:17Z
---
Review finding (2026-08-09). `→done` notes may end with an
` @ <short-sha>[+N]` anchor, and the spec calls it "a convention, not a
parse rule." But "which tree did the verify actually pass against" is
the audit field for a project whose entire differentiator is
verify-gated closure. As written, every consumer that wants it writes
its own regex, and they will disagree. Either give the anchor a column
in the `log` table or at least a normative pattern in the grammar
section — grammar-adjacent, even if extraction stays optional.

## log
- 2026-08-09T23:17Z created
