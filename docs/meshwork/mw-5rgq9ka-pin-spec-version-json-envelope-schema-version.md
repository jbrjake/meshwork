---
id: mw-5rgq9ka
title: "Pin spec version ≡ JSON envelope schema version"
category: core/format
needs: [mw-7c6svyn]
relates: [mw-5kp033j, mw-n6nvzpa]
verify: cargo test format::version_matches_envelope
seq: 270
docs:
  - FORMAT.md#configtoml
status: open
created: 2026-08-08T14:09Z
---
Review finding (2026-08-08). Two version numbers describe the same
contract: the FORMAT.md spec version (mw-n6nvzpa's config.toml marker)
and the `--json` envelope schema version (mw-5kp033j). Pin them to each
other now, while it's a one-line assert.

## log
- 2026-08-08T14:09Z created

## comments
- 2026-08-09T23:17Z [Jon Rubin] Review finding (2026-08-09) extends this: FORMAT.md never mentions the --json envelope's schema field at all — a reader sees format = 1 on disk and {"meshwork":{"version":…,"schema":…}} in output with no stated relationship. When pinning, state the mapping (or the independence) in FORMAT.md itself, not just in the assert.
