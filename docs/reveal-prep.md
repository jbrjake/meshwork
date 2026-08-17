# Reveal prep — everything short of the flip

The flip — announcing meshwork — is a one-word owner decision, not a project.
This is the checklist that makes it so (mw-78nabpd, step 4 of the stated
sequence: worklist → release → migrations → reveal). **The flip itself is NOT
this document**: publication stays a separate intentional owner decision under
the 2026-08-04 private-by-default posture. Nothing here announces anything.

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

## Already in place (verified 2026-08-17)

- [x] **Repo is public** — github.com/jbrjake/meshwork (unannounced).
- [x] **Releases** — v0.3.2 latest, binaries for darwin arm64 / linux
      arm64+x86_64 / windows x86_64; `scripts/cut-release.sh` keeps version
      stamps in lockstep (never hand-bump).
- [x] **Install paths** — plugin marketplace (`/plugin install
      meshwork@jbrjake`, resolves the newest tag: cutting a release IS the
      publish), pinned-binary ritual (README §getting it), `cargo install
      --git` for source builds.
- [x] **README front door** — what/why/demo/install, owner-passed numbers.
- [x] **60-second demo** — `./scripts/demo.sh`: the whole loop on a scratch
      repo, one command, zero network, deletes itself.
- [x] **Spec** — FORMAT.md versioned and self-contained; third parties
      implement from it, never from the binary.
- [x] **License (this repo)** — MIT, LICENSE committed.
- [x] **Self-hosting proof** — the repo's own store tracks its roadmap; the
      gate lints and primes it on every push.

## The flip's open decisions (owner-only)

- [ ] **The portfolio license call** (portfolio decision queue §8 item 6).
      meshwork itself ships MIT; the call is portfolio-wide (sazed's README
      license section, marasi's `ma-exsmbjb`) and the reveal touches how the
      portfolio presents. Decide before, or scope the reveal to meshwork alone.
- [ ] **Which docs stay private.** The reveal-adjacent analyses
      (REVIEW-fresh-eyes, DESIGN-thought-mill, STATUS/PATHS) live at the
      portfolio root, outside this repo — nothing to redact here. Store task
      files reference them as `../*.md` doc links; they dangle harmlessly for
      strangers. Confirm that is acceptable or prune the `docs:` refs.
- [ ] **crates.io yes/no.** `cargo install meshwork` vs `--git` only.
      Publishing pins the name and adds a release-ritual step (cut-release.sh
      would grow `cargo publish`); not reversible in the way a repo is.
- [ ] **Venue and text.** The demonstration-structure rule applies (lead with
      the question, show the thing): the 60-second demo + the two headline
      numbers ARE the post. Draft on request — not before the flip is called.

## Mechanical steps at flip time (each one command)

1. Gate green at HEAD: `./verify_meshwork.sh` (exit 0, observed).
2. Releases publishable: `gh release list -R jbrjake/meshwork` shows no
   drafts (a deleted-then-repushed tag flips its release to draft;
   `gh release edit <tag> --draft=false` repairs).
3. Marketplace serves the latest: the plugin resolves the newest tag —
   whatever the reveal names, cut it first with `scripts/cut-release.sh`.
4. Announce (the one word that stays the owner's).
