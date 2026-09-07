---
id: mw-psbn61z
title: "The covers: key with a pinned content hash, the covers table, and the cover verb"
category: capability/spec
needs: [mw-xvv7ck1, mw-8q0srvb]
seq: 740
verify: run cargo test e2e::cover_pins_hash
docs:
  - docs/PROPOSAL-spec-traceability.md#§-b-a-covers-edge
  - FORMAT.md#§-task-file
status: open
created: 2026-09-07T16:32Z
---
Waits additionally on the weft's WF2 first run (portfolio `po-add2zf4`'s tranche): add the
cross-repo `needs` when WF2 is minted. With R-F items F.1/F.2/F.4: `covers:` entries are
`docs:`-shaped refs plus `sha` over the anchored section (`src/docs.rs` resolves the section;
the hash is over its text), projected as a `covers` table (`gid, ref, sha, resolved`) — never an
`edges` row; `cover <task> <ref>` writes the pin, `cover --repin` re-reads; hand-written pins
without a matching hash are lint errors. FORMAT.md key + table, additive. MW-T4/T5.

## log
- 2026-09-07T16:32Z created
