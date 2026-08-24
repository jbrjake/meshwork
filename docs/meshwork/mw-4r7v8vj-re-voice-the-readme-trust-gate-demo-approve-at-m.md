---
id: mw-4r7v8vj
title: Re-voice the README trust-gate demo — approve-at-mint made its refusal transcripts impossible
status: open
category: meta/readme
discovered-from: mw-5fekg2q
docs:
  - README.md
  - DESIGN-meshwork.md#§-12b-trust-boundary
seq: 285
created: 2026-08-23T20:25Z
handoff: |
  Draft is in your working tree: README.md, uncommitted, all four proposal
  edits applied verbatim (quick-start close block, §closing-tasks
  re-voice with the gate narrated, work-loop close block, concepts trust
  bullet). Revise in place. The done-test was tightened to the
  transcript-only phrases ('approving verify for' / 'refusing unapproved
  verify for') because the new prose quotes the bare refusal phrase as
  narration — it reads green against the draft, so close will pass once
  your revision is committed. Then mw-5fekg2q and the release chain
  (mw-h4s4gka → sazed/leras sweeps) unblock. demo.sh still carries its
  no-op --approve; one word and it drops.
verify: "! grep -q \"approving verify for\" README.md && ! grep -q \"refusing unapproved verify for\" README.md"
attachments: [attachments/mw-4r7v8vj/proposal-readme-trust-gate-revoice.md]
blocked-reason:
---

Approve-at-mint (mw-2kgkn0j/mw-51x0wty, landed 2026-08-23) records this
clone's approval when a verify is authored via add --verify / set
--verify. Two README transcript sequences are now impossible as scripted:

1. Quick-start (the close --approve block): the "approving verify for
   ac-acnxdkg" preamble no longer prints — the text was CLI-authored in
   the same transcript, so the close runs it directly.
2. "closing tasks" (the sa-jt7zg9w pair): the entire "first refusal
   isn't even about the work" narrative depends on `close` refusing
   verify text the transcript itself authored two blocks earlier. That
   refusal no longer fires; the close goes straight to the honest
   objection (verify exit 1, stays open). The follow-up --approve block
   loses its "approving verify" lines the same way.

The gate demo still exists — it just needs a merge-arrival stand-in (a
hand-edit after mint, as the trust tests now construct) or a re-voiced
story. README prose is owner-voiced: proposals welcome, the words are
his. The transcript-replay guard (mw-5fekg2q) is dep-blocked on this —
its script cannot go green against blocks whose story must change.

## log
- 2026-08-23T20:25Z created
- 2026-08-24T13:43Z open→doing — claimed by claude (session_016iEafFdzwyKAtsU3AEMhaU)
- 2026-08-24T13:54Z doing→blocked — proposal attached; README words are owner-voiced — awaiting the owner's landing (which flips the verify green)
- 2026-08-24T16:09Z blocked→open

## comments
- 2026-08-24T13:09Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Owner ruling 2026-08-24: re-voiced narrative. Drop the staged-refusal story — no merge-arrival stand-in in the README. The close demo goes straight to the honest objection: verify runs (pre-approved at mint), exits 1, task stays open. The gate itself gets narrated, not demonstrated.
- 2026-08-24T13:54Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Proposal done, nothing landed in README (owner-voiced). attachments/mw-4r7v8vj/proposal-readme-trust-gate-revoice.md has the full diff: quick-start close block, §closing-tasks collapsed to one block with the gate narrated in prose, work-loop close block, concepts trust bullet, optional demo.sh flag drop. Every transcript line captured from real runs 2026-08-24 (binary @ 8d777c9), probes A-E in the appendix: mint-approved close goes straight to the honest objection; --approve is a silent no-op when already approved; hand-edited text still refuses. One staging finding: start now red-checks, so the quick-start scratch must stage stuff::thing red and fix it inside the existing '...' elision, or the start block grows a warning line.
