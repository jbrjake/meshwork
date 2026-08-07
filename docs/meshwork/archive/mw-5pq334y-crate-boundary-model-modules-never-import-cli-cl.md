---
id: mw-5pq334y
title: "Crate boundary: model modules never import cli/clap"
status: done
category: core/arch
verify: cargo test arch::model_boundary
docs:
  - REQUIREMENTS-meshwork.md#§-j-non-functional
seq: 230
created: 2026-08-06
---
parse/store/tables/write/edit are CLI-free today by accident, not by
rule. Add an enforced test (grep or compile probe) that model modules
never import crate::cli or clap, plus the one-line rule in CLAUDE.md.
Insurance, not surface: keeps a future model-crate split (embedding a
reader in another binary or UI) a Cargo.toml exercise instead of
surgery. Cheap whenever; filed from the 2026-08-06 review.

## log
- 2026-08-06 created
- 2026-08-07T00:43Z open→doing — claimed by claude
- 2026-08-07T00:46Z doing→done — verify exit 0
