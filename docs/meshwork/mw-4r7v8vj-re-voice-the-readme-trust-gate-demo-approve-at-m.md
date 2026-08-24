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
  Owner picked the re-voiced narrative 2026-08-24 (no merge-arrival
  stand-in). Remaining work: 1) draft the re-voiced trust-gate prose —
  close runs the mint-approved verify and fails honestly (exit 1, stays
  open); the merge-arrival gate is described in prose, not staged on
  screen. README words are owner-voiced: propose, never land unprompted.
  2) re-paste the two affected sequences from real runs (quick-start close
  block; the sa-jt7zg9w pair). 3) mw-5fekg2q then unblocks — its script
  spec is complete in its body plus one comment (subsequence matching for
  elided lines; scratch cargo project for the stuff::thing verify).
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

## comments
- 2026-08-24T13:09Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Owner ruling 2026-08-24: re-voiced narrative. Drop the staged-refusal story — no merge-arrival stand-in in the README. The close demo goes straight to the honest objection: verify runs (pre-approved at mint), exits 1, task stays open. The gate itself gets narrated, not demonstrated.
