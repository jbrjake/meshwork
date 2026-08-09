---
id: mw-51x0wty
title: "Authoring a verify records the authoring clone's approval (MW-E5 carve-out)"
category: core/verify
relates: [mw-hz1ezcg, mw-9rc4vs6, mw-f1x71yg]
verify: cargo test e2e::add_authored_verify_preapproved
docs:
  - DESIGN-meshwork.md#§-12b-trust-boundary
status: open
created: 2026-08-09T23:35Z
---
Field evidence (sazed, 2026-08-09). The session wrote a verify, ran
`close`, was refused ("task files arrive via merge and are untrusted"),
and re-ran with `--approve` in the same breath. The refusal reviewed
nothing: this clone authored the text seconds earlier — it never
arrived via merge, and the same agent supplies `--approve` freely.

Carve-out: `add --verify` (and `set --verify` when mw-f1x71yg lands)
records the same content-hash approval `close --approve` would, in the
gitignored `.cache/`. The §12b threat model is intact — approvals never
travel, merged files still refuse on a fresh clone. Complements
mw-hz1ezcg's bulk-bless (existing stores) with approval-at-authorship
(new tasks). Wants an owner nod on §12b wording: "trust attaches to
text approved by the operator" gains "authoring text through this
clone's own CLI is approval."

## log
- 2026-08-09T23:35Z created
