---
id: mw-xvtf5jx
title: Comment content-hash as spec-level identity
status: open
category: core/format
needs: [mw-zp1h12d]
relates: [mw-vmzg]
verify: cargo test e2e::comment_identity
docs:
  - DESIGN-meshwork.md#§-8-github-push
  - REQUIREMENTS-meshwork.md#§-k-comments-attachments
seq: 85
created: 2026-08-06
---
Owner-accepted 2026-08-06 (format-hardening review). The mirror
already computes hash(date, author, text) for idempotency markers
(§8). Promote that to THE comment identity in FORMAT.md so the mirror,
UI layers, and any replication dedup identically, and expose it as a
`hash` column on the comments table. Needs mw-zp1h12d minute stamps —
they kill the same-day-identical-text collision that makes the hash
untrustworthy today. Land before 3.3 freezes mirror idempotency
behavior (relates: mw-vmzg).

## log
- 2026-08-06 created
