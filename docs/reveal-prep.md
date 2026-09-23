# Reveal prep — everything short of the flip

The flip — announcing meshwork — is a one-word owner decision, not a project.
This is the checklist that makes it so (mw-78nabpd, step 4 of the stated
sequence: worklist → release → migrations → reveal). **The flip itself is NOT
this document**: publication stays a separate intentional owner decision.
Nothing here announces anything.

**What the reveal is for.** Adoption, and the feedback that comes with it: get
a usable tool in front of people and find out whether it attracts users who
make it better. Reputation and brand are the compounding half — a tool
strangers run is what makes later work legible. meshwork is not held as
defensible IP, so copyability is not a consideration in any decision on this
page; the gate on the flip is that the tool is good enough to keep the users it
attracts, which is why v0.5.0 carries the fixes real use turned up
(`mw-59f0t1q`).

## The headline the reveal ships with

From [`setup-cost-matrix.md`](setup-cost-matrix.md) (the store's own logs,
denominators inside) and the README's measured numbers:

- Session-start onboarding drops **31×** (28K → ~940 tokens); compounded
  busywork **4.2× less** (4.19M → 0.99M tokens/session) — README §numbers.
- An agent session reaches its first task action in a median **9.1 min**
  carrying **~102k tokens** of context (n=87 sessions) — and a **cross-repo
  switch costs nothing extra** (8.8 min vs 11.1 same-repo): the store carries
  the context the context window drops. The first empirical numbers on agent
  context-switch cost, measurable because the store keeps what every other
  setup throws away.

## Already in place

- [x] **Repo is public** — github.com/jbrjake/meshwork (unannounced).
- [x] **Releases** — binaries for darwin arm64 / linux arm64+x86_64 / windows
      x86_64; `scripts/cut-release.sh` keeps version stamps in lockstep (never
      hand-bump). The reveal names the v0.5.0 cut.
- [x] **Distribution** — the Claude Code plugin marketplace is the channel:
      `/plugin install meshwork@jbrjake` resolves the newest tag, so cutting a
      release IS the publish. Pinned-binary ritual (README §getting it) and
      `cargo install --git` are the secondary paths.
- [x] **README front door** — what/why/demo/install, owner-passed numbers.
- [x] **60-second demo** — `./scripts/demo.sh`: the whole loop on a scratch
      repo, one command, zero network, deletes itself.
- [x] **Spec** — FORMAT.md versioned and self-contained; third parties
      implement from it, never from the binary.
- [x] **License (this repo)** — MIT, LICENSE committed.
- [x] **Self-hosting proof** — the repo's own store tracks its roadmap; the
      gate lints and primes it on every push.

## Settled

- **Licensing.** meshwork ships MIT, LICENSE committed. The portfolio-wide
  call was STRUCK by owner ruling 2026-08-21 (`portfolio#po-g55b2d1`), interim
  posture of record: *"nothing goes public that the owner wouldn't share as
  MIT."* The reveal needs nothing further here.
- **crates.io.** Not the channel and not a question: distribution is the
  Claude Code plugin marketplace, where a cut release publishes itself.

## The flip's open decisions (owner-only)

- [ ] **Which docs stay private.** The reveal-adjacent analyses
      (REVIEW-fresh-eyes, DESIGN-thought-mill, STATUS/PATHS) live at the
      portfolio root, outside this repo — nothing to redact here. Store task
      files reference them as `../*.md` doc links; they dangle harmlessly for
      strangers. Confirm that is acceptable or prune the `docs:` refs.
- [ ] **Venue and text.** The demonstration-structure rule applies (lead with
      the question, show the thing): the 60-second demo + the two headline
      numbers ARE the post. Draft on request — not before the flip is called.

## Mechanical steps at flip time (each one command)

0. `mw-59f0t1q`'s three `needs:` closed — the v0.5.0 scope. The plan/m3 and
   plan/m4 mirror block is deferred and outside it.
1. Gate green at HEAD: `./verify_meshwork.sh` (exit 0, observed).
2. Push. The local commits ahead of `origin` are the v0.5.0 work in flight;
   the remote is the artifact strangers read, so it carries the cut.
3. Cut v0.5.0 with `scripts/cut-release.sh` (`docs/release-notes/RELEASE-NOTES-v0.5.0.md`
   is a precondition of the cut). The marketplace plugin resolves the newest
   tag, so this is also the publish.
4. Releases publishable: `gh release list -R jbrjake/meshwork` shows no
   drafts (a deleted-then-repushed tag flips its release to draft;
   `gh release edit <tag> --draft=false` repairs).
5. Announce (the one word that stays the owner's).
