---
id: mw-ps4fzn2
title: Let run cargo test scope to a crate and a test target with dash-free package= and target= tokens
category: core/verify
needs: [mw-t41d6ze]
answers: marasi#ma-tpmcdnk
seq: 530
verify: run cargo test verify_dsl::run_package_target_tokens
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-2-5-defects
  - docs/DESIGN-meshwork.md#§-12b-trust-boundary
status: open
created: 2026-09-07T16:32Z
---
The runner grammar forbids a leading dash, correctly, so `-p <crate>` is inexpressible and
every `run cargo test <filter>` compiles the workspace: 18 minutes against a 5-minute timeout
in leras, 43 of 58 verifies kept on shell for this. With R-E item E.1: `package=<crate>` and
`target=<name>` (tight class, `=` already legal) are translated by meshwork into `-p`/`--test`;
no author text becomes a flag. Grammar comment in `src/verify_dsl.rs`, the `verify --help`
text, and the DSL fixture corpus.

## log
- 2026-09-07T16:32Z created
