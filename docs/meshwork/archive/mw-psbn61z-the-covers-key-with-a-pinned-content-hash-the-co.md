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
status: done
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
- 2026-09-22T14:20Z open→done — verify exit 0 @ d78dd23+4

## comments
- 2026-09-21T00:06Z [claude (6104c7c3-6cf9-4c90-90ac-e4ec66924a6d)] R-F ruled 2026-09-20 (mw-xvv7ck1) and two premises in this body no longer hold.
  
  F.4 went the other way: clause refs get their OWN syntax, not `docs:`-shaped refs. A pin must
  survive a heading edit, which a slug does not, and the `docs:` resolver carries a live defect
  (sazed#sa-rj7vxkt: `anchor-missing` on nine tasks whose GitHub slugs are correct — emoji and
  em-dashes each collapse to a double hyphen). So `src/docs.rs` is not the resolver for this; the
  format gains a second pointer spelling deliberately. REQUIREMENTS MW-T2 is the clause.
  
  And the lane is no longer gated on the weft's WF2 first run — do not add that cross-repo `needs`.
  `portfolio#po-add2zf4` is itself unpromoted, so gating on it was a deferral chain, not a
  measurement. Build it. MW-T6: the hash is taken over clause text read from disk, so a pin detects
  drift in an unversioned corpus such as `gestalt/` (still not a git repo); what version control
  would add is attribution of the change, not detection of it.
