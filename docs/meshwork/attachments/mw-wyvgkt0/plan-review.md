# Review of docs/PLAN-demo-notes.md against the source

Every claim below was checked in the meshwork source at 76541b1 before any demo repo existed.

## Defects

1. **Beat 3's audit expects one re-open candidate and gets two.** S4 (done at day 0) pins `docs/PROTOCOL.md#sp-conflict-resolution`, and v0.2.0 rewrites that clause. `Audit::drifted(false)` (src/spec.rs) lists every done task whose pin moved, so `re-open candidates` prints S4 and LABEL, and the replay's `expect` fails. Fix: in Beat 2, sync-1 runs `spec audit docs/PROTOCOL.md`, re-reads the conflict clause and runs `cover $S4 --repin`; `$ANSWER` also covers `sp-conflict-resolution` and `sp-observed-at`. cover.rs puts no status limit on a repin and `Located::write` reaches bundled archives, but no probe has repinned a done task yet. Add that probe to the plan's list.
2. **The re-enactment reads files nobody commits.** `reports/gate-rewrite/` stays untracked by design. Once patch 3a drops the `#[ignore]`, `cargo test` on cli main fails in CI and in every clone at `story/3-resolved`. Fix: patch 1a commits the two logs under `tests/notes/fixtures/gate-rewrite/` and the test reads them there.
3. **Patches cannot carry runtime ids.** Ids mint from entropy (src/id.rs), so every replay mints ids the recording never had. Step 19 writes `#[ignore = "waits on <$ASK>"]` and patch 3a deletes those lines, so a static patch with the recorded id never applies in demo mode. Fix: patches carry placeholders that the `patch` helper substitutes before `git apply`. The alternative is `MESHWORK_ID_SEED`, the binary's deterministic-id test hook, for recording and replay alike.
4. **Demo-mode clones already hold the future.** A full clone of sync has `v0.2.0` and every `story/*` tag, and main sits past `story/0-day0`. Checking out the tag leaves a detached HEAD, and Beat 2's `git tag v0.2.0` fails. Fix: after cloning, `git checkout -B main story/0-day0` and delete the later tags locally. Also state which notesync Beat 3 builds in demo mode. Today the cargo git dep fetches the recorded v0.2.0 from GitHub, not the replay's own Beat 2. A scratch `[patch]` in a `.cargo/config.toml` above the clones would point it at the sibling, and cargo reads that config in verify runs too.
5. **S8's verify is green from S1.** The layout scaffolds `version 0.1.0`, so `contains Cargo.toml /^version = "0\.1\.0"/` passes before any release work, and `start` reports it already green. Fix: S8 checks something only the release writes, e.g. `contains CHANGELOG.md /^## 0\.1\.0/`.
6. **The cli's day 0 needs the demo registry.** The work breakdown says the cli depends only on `v0.1.0`. But LABEL's `cover meshwork-demo-notes-sync#docs/PROTOCOL.md#sp-change-timestamp` hashes the sibling's clause through the registry, and `add --needs meshwork-demo-notes-sync#…` for TITLE warns when the registry lacks the repo (src/cli/add.rs). The default registry is ~/Documents/code/portfolio. It does not register the demo repos, and registering them would put their prop backlogs in the real `portfolio ready`. Fix: the cli depends on R1, and builder sessions in the demo repos export `MESHWORK_PORTFOLIO` at the demo portfolio checkout. Registered paths default to ~/Documents/code/<name>.
7. **Re-recording wipes R7's cli and sync READMEs.** R7 lands after the recording, and a re-record force-pushes those repos' main back to `story/0-day0`. Fix: the staged disclosure and the replay pointer land as a day-0 task in each of those two repos. Only the portfolio README follows R6.

## Smaller

- Beat 1 step 13: `portfolio search merge` finds TITLE only if its body says merge; its title does not. Give TITLE a body naming per-field merge.
- `story/fixture-gen` is a cargo crate, so it gets the portfolio's Rust gate scaffold like the other two.
- v0.2.0 keeps `Doc::timestamp_ms`, now the HLC's ms. Otherwise patch 3a (tag bump only) does not compile and the label bug never shows.
- Beat 3 step 4: build once after 3a so the v0.2.0 fetch and the Cargo.lock update happen in the replay. Otherwise they happen inside the verify runner, which keeps only PATH, HOME, CARGO_HOME and TMPDIR (src/verify_exec.rs).
- The fixture's T is its generation time minus four hours. Regenerate it at record time so the reported flight sits hours, not days, before the recorded sessions.
- Builders' checkouts sit under ~/Documents, and verify builds ignore `CARGO_TARGET_DIR`, so `mdutil -i off` each demo checkout (portfolio Rust rule 4).
- `.meshwork-version` sits outside `docs/meshwork/`. A commit carrying the plugin hook's pin rewrite with a task file makes that task ride-along under provenance and gates its `run` verifies. Pin rewrites commit alone.
- Fences that open after a list marker (`7. ```` and the like) hide every heading from Beat 3 on from meshwork's anchor scan, a meshwork bug filed on its own. Until it lands, docs refs to those sections warn. Moving each fence onto its own line clears them now.

## Checked and sound

Leave these as written:
- the prime strings the beats assert: `spec moved under 1 live task`, `[claimed: …]`, `addressed to this repo (1):`, `asks out`;
- P2's diagnosis: prime injects only terminal foreign rows (`query::terminal_foreign`), so an open registered target counts as unresolved;
- an ignored test reads red under the `ok. N passed` floor;
- lint's spec-drift skips done tasks, and lint warnings exit 0;
- close ignores another session's claim;
- reopen extracts a bundled task;
- the epilogue's columns exist.
