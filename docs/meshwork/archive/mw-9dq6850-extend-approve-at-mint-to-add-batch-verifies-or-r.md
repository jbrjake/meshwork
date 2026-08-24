---
id: mw-9dq6850
title: "Extend approve-at-mint to add --batch verifies, or rule the carve-out"
status: done
category: core/trust
discovered-from: mw-4r7v8vj
verify: run cargo test batch_authored_verify_preapproved
seq: 240
created: 2026-08-24T16:42Z
---

Found reviewing mw-4r7v8vj's README security prose (2026-08-24): a verify
carried in an add --batch document is NOT mint-approved — close refuses it
(probed on a scratch store, binary @ 5d04b00: batch-minted `verify: true`
→ "refusing unapproved verify"). Only add --verify and set --verify record
the mint approval (e2e_verify_trust.rs names exactly those two; no batch
case exists in the suite), so the gap is untested and undocumented either
way — accident or carve-out, nobody ruled.

The case for extending: the batch document is authored through this
clone's CLI by the same operator the flag path trusts, and --dry-run
exists precisely to review it before minting. The case for the carve-out:
batch input arrives as a file/stdin stream, which is also how untrusted
content moves. Owner rules.

If extend: mint approvals for each batch document's verify at write, same
(id, text) ledger entry as the flag paths; the verify names the test to
land (batch sibling of add_authored_verify_preapproved). If carve-out:
re-shape this verify to the doc change — README's "task verification
security" section and the skill's batching guidance must both say batch
verifies gate at close like any file arrival.

README impact today (mw-4r7v8vj draft, pre-land): "When you add a task in
meshwork and provide the verify field, it's trusted" over-claims — the
README's own #### batching section adds verify-carrying documents whose
closes would refuse. Wording fix proposed on mw-4r7v8vj.

## log
- 2026-08-24T16:42Z created
- 2026-08-24T16:56Z open→doing — claimed by claude (session_016iEafFdzwyKAtsU3AEMhaU)
- 2026-08-24T17:12Z doing→done — verify exit 0 @ 940e387+2

## comments
- 2026-08-24T16:56Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Owner ruling 2026-08-24: extend — batch-minted verifies get the same at-write approval as add --verify/set --verify. The README security prose landed as-is (112e32e) on the strength of this fix making it accurate.
