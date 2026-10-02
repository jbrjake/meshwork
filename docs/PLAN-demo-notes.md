# PLAN — the notes demo

A story performed on three real, public repos that use meshwork from their first commit. This is the full demo and it needs the network. It sits beside the small offline demo, `scripts/demo.sh`, which stays a one-repo scratch loop. The demo shows meshwork as released, the version the repos pin; it requires no change to meshwork. This is a plan: nothing in it is built.

## The story

A notes app syncs through a replication library that the app does not own. One user message reports a lost edit. Three agent sessions that never meet trace it to the library's conflict resolution, get the library changed by asking, and catch the consequence that change has for the app. The only human input is that first message.

- **The report** (to a fresh session in the app repo; the one human message, verbatim):
  > At the gate I fixed a typo in a note on my laptop and let both devices sync. After takeoff I rewrote the note on my phone. When I landed and synced, my rewrite was gone and the laptop's version won. Both devices' change logs are in reports/gate-rewrite/.
- **notes-1** opens a case and re-enacts the report as a test. Seeing that the app's sync pushes before it pulls, it declares a smoking gun, reorders the sync, and tries to close the case. meshwork refuses: the re-enactment still fails. Reading the logs, it finds the real cause. The phone stored the laptop's typo fix *before* writing its rewrite, yet the rewrite carries the earlier timestamp: the laptop's clock runs about five minutes fast. Last-writer-wins on wall-clock time cannot see that the rewrite came after. That ordering is the library's protocol, so notes-1 files an ask to the library repo and pins the protocol clause it is asking to change. It checks the library's roadmap first (per-field merge is planned, but both edits replaced the body, so it would not help). It leaves a handoff on the ask and ends.
- **sync-1**, a clean session in the library repo, finds the ask in its own queue. It answers by stamping changes with hybrid logical clocks (HLC), adds a local `observed_at`, rewrites the protocol clause, and releases v0.2.0.
- **notes-2**, a clean session in the app repo, opens on a digest that lists the ask as answered and warns that the pinned clause moved. `show` on the ask gives it notes-1's handoff. It re-reads the clause: an HLC timestamp "can run ahead of any device's clock". A portfolio-wide spec audit names a task closed on day 0, the "edited N minutes ago" label, as built on the old sentence. Under HLC the phone's own rewrite now reads "edited in 5 minutes". notes-2 moves to v0.2.0, closes the ask, reopens the label task, fixes it to read `observed_at`, and closes the case.
- **Epilogue**: one query over the portfolio shows the whole story as log rows and comments, including the smoking gun and its retraction.

The plot is credible because:
- LWW on device clocks is a deliberate simplification many real systems ship, with a documented skew caveat.
- HLC is a protocol and on-disk change that serves every consumer of the library. It is roadmap work, not a patch the app should carry.
- HLC fixes exactly the causal case reported, the edit made after seeing the other. The protocol text says plainly that truly concurrent edits still race.

## Settled decisions

| Decision | Choice |
|---|---|
| meshwork | The pinned release, v0.5.2, unchanged. The demo shows what ships. |
| Plot | Lost offline edit from LWW conflict resolution on device wall clocks |
| Repos | `jbrjake/meshwork-demo-notes-cli`, `jbrjake/meshwork-demo-notes-sync`, `jbrjake/meshwork-demo-notes-portfolio`, all public |
| Performance | Staged: one script performs every session with real meshwork commands, real code changes and real outputs. The sessions are scripted, and each repo's README says so. |
| Timestamps | Real. Store log lines carry the time the recording ran. |
| Human input | Exactly one message, the report above |
| Two demos | `scripts/demo.sh` stays small and offline: the quick-start loop on a scratch repo. `scripts/demo-full.sh` runs this story from the public repos and needs the network. |

These follow from those choices:
- **Language: Rust, std only.**
  - meshwork's verify DSL runs only `cargo`.
  - A `run cargo test` verify needs no approval while the task file's history is store-only. The story has no human to approve anything.
  - Std only means no registry fetches and builds in seconds.
- **The app depends on the library by git tag** (`tag = "v0.1.0"`, later `"v0.2.0"`), the way separate projects consume each other. The library's PROTOCOL.md is read from the sibling checkout through the portfolio registry.
- **Registry names equal repo names.** Cross-repo refs read `meshwork-demo-notes-sync#sy-…`. That form is proven by probe (below); short aliases are not.
- **Every demo session resolves through the demo registry.** Sessions working in the demo repos export `MESHWORK_PORTFOLIO=~/Documents/code/meshwork-demo-notes-portfolio`, and the checkouts sit at `~/Documents/code/<name>`, the registry's default path. The real portfolio does not register the demo repos; registering them would put their prop backlogs in the real `portfolio ready`.
- **Aliases are set in `config.toml` before the first `add`.** `init` would derive `me` for all three.

  | Repo | Alias |
  |---|---|
  | sync | `sy` |
  | cli | `nt` |
  | portfolio | `pf` |

- **Session identities:** `MESHWORK_AUTHOR` is `claude (notes-1)`, `claude (sync-1)` or `claude (notes-2)`. Git commits use the repo owner's identity with a `Staged-session: <name>` trailer.
- **Commit discipline:**
  - Conventional Commits.
  - Task-file changes always go in their own commits, apart from code, which keeps `run` verifies approval-free.
  - A `.meshwork-version` rewrite (the plugin's session-start hook makes them) always commits alone. The pin sits outside `docs/meshwork/`, so riding with a task file would gate that task's `run` verifies under provenance.
  - Tests that wait on the ask are committed `#[ignore = "waits on <ask id>"]`, so `main` stays green. An ignored test reads as red to the verify (the `ok. N passed` floor), which is what the tasks need.

Probed in scratch stores before this plan was written:
- the case tree;
- `why` across repos;
- the ask surfacing in the addressee's `ready`;
- `--answers`;
- live `spec-drift` on a cross-repo pin;
- the day-0 task listed under `re-open candidates` in `portfolio spec audit`;
- `cover --repin`.

Probed on v0.5.2 itself: once the answer is done, the asker's prime lists the ask under `asks out` with `answered-by <gid> (done)`, and a started case shows as `doing <id> <title> [claimed: <author>]`.

Not yet probed: `cover --repin` on a done, archived task, which Beat 2 runs on S4. cover.rs puts no status limit on a repin and writes into bundled archives.

**The binary.** The three repos pin `.meshwork-version` to v0.5.2. The recording and the replay run that release.

## Repo: meshwork-demo-notes-sync

The generic document-replication core. Its API speaks documents and fields, never notes. Its README names it as the replication layer for any app that syncs documents between devices. Transport is the caller's concern.

**Layout**
```
Cargo.toml                package meshwork-demo-notes-sync, [lib] name = "notesync", version 0.1.0; [profile.dev] scaffold
rust-toolchain.toml       channel = "1.97.0" (matches meshwork)
.cargo/config.toml        [build] rustflags/rustdocflags = ["-D", "warnings"]
src/lib.rs                re-exports
src/clock.rs              Clock trait, SystemClock, ManualClock
src/change.rs             Change, Fields, Doc
src/log.rs                append-only change log: encode/decode, escaping
src/replica.rs            Replica: open, put, doc, docs, seen, changes_since, apply
src/merge.rs              conflict resolution
src/vv.rs                 VersionVector
tests/notesync/main.rs    the one test target; topics as mods (log, merge, sync, clock)
docs/PROTOCOL.md          the contract, as {#sp-…} clauses
README.md, CHANGELOG.md
.github/workflows/ci.yml  fmt, clippy, test; actions pinned by commit SHA
docs/meshwork/            store (alias sy), .meshwork-version, shim
```

**API at v0.1.0**
```rust
pub trait Clock { fn now_ms(&self) -> u64; }
pub struct SystemClock;
pub struct ManualClock { /* Cell<u64> */ }          // new(ms), set(ms), advance(ms)
pub type Fields = std::collections::BTreeMap<String, String>;
pub struct Change { pub device: String, pub seq: u64, pub timestamp_ms: u64, pub doc: String, pub fields: Fields }
pub struct Doc { pub id: String, pub fields: Fields, pub timestamp_ms: u64, pub device: String }
pub struct VersionVector(/* BTreeMap<device, highest seq> */);
impl<C: Clock> Replica<C> {
    pub fn open(dir: &Path, clock: C) -> io::Result<Self>;   // device id = dir's file name, kept in <dir>/device
    pub fn device(&self) -> &str;
    pub fn put(&mut self, doc: &str, fields: Fields) -> io::Result<Change>;
    pub fn doc(&self, id: &str) -> Option<&Doc>;
    pub fn docs(&self) -> impl Iterator<Item = &Doc>;
    pub fn seen(&self) -> VersionVector;
    pub fn changes_since(&self, seen: &VersionVector) -> Vec<Change>;
    pub fn apply(&mut self, changes: Vec<Change>) -> io::Result<usize>; // idempotent; returns how many were new
}
```

**On disk at v0.1.0**
- `<dir>/device` holds the device id.
- `<dir>/changes.log` holds one change per line, appended in the order this replica stored it, whether authored or received. Log order is therefore local causal order; notes-1's diagnosis reads it.
- Columns, tab-separated: `device`, `seq`, `timestamp_ms`, `doc`, `fields`.
- `fields` is `key=value` pairs joined by `;`. Tab, newline, backslash, `;` and `=` are backslash-escaped.

**docs/PROTOCOL.md at v0.1.0** (clause text exact; the pins hash these sections):
```
# notesync protocol

## Changes {#sp-changes}
A change replaces one whole document: its id and every field, with the device that made it, that device's sequence number, and a timestamp.

## Change timestamps {#sp-change-timestamp}
A change's timestamp is the authoring device's wall-clock time when the change was made, in milliseconds since the Unix epoch.

## Conflict resolution {#sp-conflict-resolution}
When two changes replace the same document, the one with the higher timestamp wins; equal timestamps break on device id. Resolution is order-independent: replicas that have stored the same changes agree. Device clocks are trusted, so a device whose clock runs fast wins conflicts it should lose.

## Sync {#sp-sync}
Replicas exchange the changes the other has not seen, by version vector. Moving changes between replicas is the caller's concern.
```

The README carries the same caveat in plain words: conflict resolution trusts device clocks, so keep them synced.

**Day-0 tests:**
- `log_round_trips`, `log_escapes_separators`
- `lww_higher_timestamp_wins`, `lww_tie_breaks_on_device`, `lww_is_order_independent`
- `sync_exchanges_missing_changes`, `sync_apply_is_idempotent`
- `manual_clock_advances`

**Day-0 tasks.** Filed in this repo's store as the crate is built, each closed by its verify.

| # | Title | Verify | Pin |
|---|---|---|---|
| S1 | Scaffold the crate with the portfolio's Rust gate scaffold | `all(contains Cargo.toml /^\[profile\.dev\]/, exists .cargo/config.toml, exists rust-toolchain.toml, exists tests/notesync/main.rs)` | |
| S2 | Write the protocol the library promises | `all(contains docs/PROTOCOL.md /sp-change-timestamp/, contains docs/PROTOCOL.md /sp-conflict-resolution/)` | |
| S3 | Persist each replica's changes as an append-only log | `run cargo test log_` | |
| S4 | Resolve conflicting changes last-writer-wins by timestamp | `run cargo test lww_` | `docs/PROTOCOL.md#sp-conflict-resolution` |
| S5 | Exchange missing changes between replicas by version vector | `run cargo test sync_` | `docs/PROTOCOL.md#sp-sync` |
| S6 | Say in the README that conflict resolution trusts device clocks | `contains README.md /trusts device clocks/` | |
| S7 | Run fmt, clippy and tests in CI with actions pinned by SHA | `exists .github/workflows/ci.yml` | |
| S8 | Release v0.1.0 | `contains CHANGELOG.md /^## 0\.1\.0/` | |
| S9 | Say in the README that the sessions are staged and how to replay them | `contains README.md /story\/replay\.sh/` | |

S8 closes with tag `v0.1.0`. Its verify reads the CHANGELOG entry only the release writes, since the scaffold already carries `version = "0.1.0"`. S9 lands at day 0 because a re-record force-pushes main back to `story/0-day0`, which would wipe a README commit made after the story.

**Open backlog at day 0:**

| # | Title | Seq | Verify |
|---|---|---|---|
| PFM | Merge per field so concurrent edits to different fields both survive | 10 | `run cargo test per_field_merge` |
| | Compact the change log once every known replica has seen a change | 20 | `run cargo test compact_` |

## Repo: meshwork-demo-notes-cli

A notes CLI over the library. A device is a folder: a laptop and a phone are two directories.

**Layout**
```
Cargo.toml                package meshwork-demo-notes-cli, [lib] name = "notes", [[bin]] name = "notes"
                          notesync = { package = "meshwork-demo-notes-sync", git = "https://github.com/jbrjake/meshwork-demo-notes-sync", tag = "v0.1.0" }
Cargo.lock                committed
rust-toolchain.toml, .cargo/config.toml, [profile.dev]   as in sync
src/main.rs               thin: parse args, call notes::cli::run
src/cli.rs                notes [--device DIR] new TITLE [--body TEXT|-] | edit ID [--title T] [--body TEXT|-] | list | show ID | sync OTHER_DIR
                          default device: $NOTES_DEVICE, else ./device
src/note.rs               Note { id, title, body } <-> notesync Fields; note id = "<device>-<seq>" of its first change
src/device.rs             open a device folder as Replica<SystemClock>
src/sync.rs               folder-to-folder sync: push local changes the remote lacks, then pull
src/label.rs              edited_ago(now_ms, at_ms) -> "edited just now" | "edited N minutes ago" | "… hours ago" | "… days ago"
                          | "edited in N minutes" when at_ms is ahead of now
tests/notes/main.rs       the one test target; mods device, cli, sync, label, report, contract
docs/meshwork/            store (alias nt), .meshwork-version, shim
README.md, .github/workflows/ci.yml (SHA-pinned)
```

At day 0, `list` labels each note with `edited_ago(now, doc.timestamp_ms)`. This is the line HLC breaks.

**Day-0 tests:**
- `device_store_round_trips`
- `cli_new_edit_list_show`
- `sync_two_devices_converge`
- `edited_label_counts_minutes`, `edited_label_hours_and_days`

**Day-0 tasks:**

| # | Title | Verify | Pin |
|---|---|---|---|
| C1 | Scaffold the crate with the portfolio's Rust gate scaffold | as S1, path `tests/notes/main.rs` | |
| C2 | Store notes as notesync documents in a device folder | `run cargo test device_` | |
| C3 | Add new, edit, list and show | `run cargo test cli_` | |
| C4 | Sync two device folders | `run cargo test sync_two_devices` | |
| LABEL | Show when each note was last edited in list | `run cargo test edited_label` | `meshwork-demo-notes-sync#docs/PROTOCOL.md#sp-change-timestamp` |
| C6 | Run fmt, clippy and tests in CI with actions pinned by SHA | `exists .github/workflows/ci.yml` | |
| C7 | Say in the README that the sessions are staged and how to replay them | `contains README.md /story\/replay\.sh/` | |

LABEL's `cover` hashes the sibling's clause through the demo registry, and TITLE's cross-repo `needs` is checked against it, so the cli's day 0 runs after R1. C7 lands at day 0 for the same reason as S9.

The LABEL pin is what drifts. Its verify filter, `edited_label`, also matches the test notes-2 adds (`edited_label_never_reads_ahead`), so the day-0 verify grows teeth without a `set --verify`. LABEL stays unranked so its reopen collides with no seq.

**Open backlog at day 0:**

| # | Title | Seq | Needs | Verify |
|---|---|---|---|---|
| TITLE | Keep both title edits when two devices rename a note concurrently | 30 | `meshwork-demo-notes-sync#<PFM id>` | `run cargo test concurrent_title_edits` |
| | Export a note as markdown | 40 | | `run cargo test export_markdown` |
| | Search notes by text | 50 | | `run cargo test search_` |

TITLE is the link to already-planned work in the other repo, the one notes-1 checks with `why`. Its body names notesync's per-field merge; its title does not say merge, and Beat 1's `portfolio search merge` finds it through the body.

## Repo: meshwork-demo-notes-portfolio

The registry plus the story machinery. It registers itself, as the real portfolio does, and keeps its own store for portfolio-scope work.

```
repos.toml               [[repo]] meshwork-demo-notes-portfolio, meshwork-demo-notes-cli, meshwork-demo-notes-sync — https remotes
.gitignore               repos.local.toml
docs/meshwork/           store (alias pf)
README.md                what the three repos are; the story is staged; every command and output is real; how to replay
story/fixture-gen/       tiny crate on notesync v0.1.0 (git tag) that writes the report fixture; Rust gate scaffold as in sync
story/fixtures/gate-rewrite/laptop-changes.log, phone-changes.log
story/patches/           one patch per code step, named <beat><step>-<repo>-<slug>.patch; ids appear only as placeholders
story/text/              task bodies, comments and handoffs the sessions write (read via @file)
story/replay.sh          performs the story; see "The replay"
story/recording.md       written by --record: when it ran, the meshwork version used, the tags it pushed
```

**The fixture.** `fixture-gen` drives two replicas with `ManualClock`s and keeps the real times relative to its run. Let T be the generation time minus four hours. The laptop's clock reads true + 5 minutes; the phone's reads true.

1. At T, the laptop creates "Keynote outline" (body v1), and the two devices sync.
2. At T+60m, the laptop fixes a typo (body v2, stamped T+65m), and the two devices sync. The phone stores v2.
3. At T+63m, the phone rewrites the note (body v3, stamped T+63m). It is in the air, offline.
4. At T+180m, after landing, the two devices sync. v2 wins on both.

The fixture is both change logs after step 4. The phone's log shows v2 stored before v3, while v3 carries the lower stamp. `--record` regenerates and commits it before Beat 1, so the reported flight sits hours, not days, before the recorded sessions. The default mode replays the committed fixture.

## Beats

Each beat is one staged session. Commands run through the real binary; `$X` is an id minted earlier in the replay. Every beat ends with `meshwork lint` exiting 0 in each touched repo.

### Beat 0 — day 0

The three repos as built: tasks S1–S9, C1–C7 and LABEL closed, backlogs open, sync tagged `v0.1.0`. The cli and sync repos are tagged `story/0-day0`.

The replay opens with `meshwork portfolio ready` as the establishing shot: what each repo has on its plate.

### Beat 1 — notes-1 (meshwork-demo-notes-cli)

Author `claude (notes-1)`.

**Open the case**
1. `meshwork prime` — what the session-start hook injects.
2. The report prints, labeled as the only human input. The two logs appear in `reports/gate-rewrite/`, uncommitted, as the user left them.
3. Add the case:
   ```
   meshwork add "Stop edits made after a sync losing to older ones" --cat sync --seq 10 --body @case.md --verify "run cargo test reported_gate_rewrite_survives"
   ```
   This mints `$CASE`.
4. `meshwork attach $CASE reports/gate-rewrite/laptop-changes.log`, then the same for the phone's log.
5. `mkdir -p tests/notes/fixtures/gate-rewrite && cp reports/gate-rewrite/*.log tests/notes/fixtures/gate-rewrite/`, so the evidence the test reads is committed; `reports/` stays the user's, untracked. Then patch `1a-cli-reenact`: `tests/notes/report.rs`. It reads the two logs from `tests/notes/fixtures/gate-rewrite/`, replays the logged operations on fresh replicas whose clocks reproduce each device's recorded readings, syncing where the logs show a sync, and asserts both devices end on v3. Show the diffstat.
6. `meshwork start $CASE` — the red-check runs the test, which fails, as it should.

**The smoking gun, refused**
7. Comment on the case:
   ```
   meshwork comment $CASE "Smoking gun: notes sync pushes before it pulls, so the laptop's typo fix lands on top of the rewrite. Pulling first."
   ```
8. Patch `1b-cli-pull-first` (`src/sync.rs`). Show the diff, about 6 lines.
9. `meshwork close $CASE` — **refused**: it stays doing, because the verify failed. The replay asserts the refusal.
10. `git checkout -- src/sync.rs`.

**The real cause**
11. `cut -f1-3 reports/gate-rewrite/phone-changes.log` — the phone stored the laptop's v2 before writing v3, and v3's stamp is lower.
12. Retract it:
    ```
    meshwork comment $CASE @diagnosis.md
    ```
    The comment text: "Wrong: notesync merges last-writer-wins on timestamps, so push/pull order cannot matter. The phone stored the laptop's typo fix before it wrote the rewrite, but the laptop's clock runs about five minutes fast, so the fix carries the later stamp and wins. Ordering is notesync's protocol, not ours."
13. `meshwork portfolio search merge` finds sync's per-field merge and our TITLE. `meshwork why $TITLE` shows it blocked by `meshwork-demo-notes-sync#$PFM (open)`. Per-field merge would not help, since both edits replaced the body.

**Ask the library**
14. Add the ask:
    ```
    meshwork add "Order changes so an edit always beats the changes its author had already seen" --to meshwork-demo-notes-sync --parent $CASE --seq 20 --body @ask.md --verify "run cargo test causal_order_survives_fast_clock"
    ```
    This mints `$ASK`.
15. `meshwork cover $ASK meshwork-demo-notes-sync#docs/PROTOCOL.md#sp-change-timestamp`.
16. Patch `1c-cli-contract`: `tests/notes/contract.rs`. A replica with a +5-minute clock writes; the other replica syncs and then writes; the second write must win on both. Red at v0.1.0.
17. `meshwork tree $CASE`. Then `meshwork ready` shows the ask under `asks out`.
18. `meshwork set $ASK --handoff @handoff.md`. The handoff text: "Not sync order: pull-first changed nothing and the close was refused (comments on the case). The laptop's clock runs about five minutes fast and LWW on wall-clock time let its typo fix beat a rewrite made after it; the evidence is attached to the case. When notesync answers, move to its release; the contract test and the case's re-enactment should both go green."

**Commit and tag**
19. Patch `1d-cli-wait-on-ask` marks `reported_gate_rewrite_survives` and `causal_order_survives_fast_clock` with `#[ignore = "waits on @ASK@"]`; the `patch` helper writes `$ASK` in place of the placeholder.
20. Commit the store: `chore(store): open the gate-rewrite case and ask notesync for causal ordering`.
21. Commit the tests and their fixture: `test(sync): re-enact the reported gate rewrite and pin the causal-order contract`.
22. Tag `story/1-report`.

### Beat 2 — sync-1 (meshwork-demo-notes-sync)

Author `claude (sync-1)`.

1. `meshwork prime` — `addressed to this repo (1): meshwork-demo-notes-cli#$ASK …`. Its next is PFM.
2. `meshwork show $ASK`, run in the cli repo: the ask's body and pin.
3. Add the answer:
   ```
   meshwork add "Stamp changes with hybrid logical clocks" --cat protocol --seq 5 --answers meshwork-demo-notes-cli#$ASK --verify "all(run cargo test change_sorts_after_everything_its_author_saw, run cargo test v1_log_reads_with_counter_zero, run cargo test observed_at_is_local)"
   ```
   This mints `$ANSWER`.
4. Patch `2a-sync-hlc-tests`, then `meshwork start $ANSWER` (red).
5. Patch `2b-sync-hlc`. It contains:
   - **HLC:** `Timestamp { ms, counter }`. `put` stamps max(local wall, newest seen) with a counter, and `apply` advances the newest-seen stamp.
   - **`observed_at`:** `Doc::observed_at_ms()`, local and never sent. It is a new local log column, and v0.1 lines fall back to their timestamp.
   - **v0.1 compatibility:** v0.1 log lines read with counter 0.
   - **API:** `Change` and `Doc` keep `timestamp_ms`, now the HLC reading's ms, and gain a `counter` field beside it. The cli therefore compiles unchanged at patch 3a, and its label keeps reading the HLC ms, which is the bug notes-2 finds.
   - **Docs and version:** the PROTOCOL.md changes below, the README caveat, CHANGELOG, and version 0.2.0.

   The replay shows `git diff -- docs/PROTOCOL.md`; that diff is the beat's centerpiece.
6. `meshwork cover $ANSWER docs/PROTOCOL.md#sp-change-timestamp`, then the same for `#sp-conflict-resolution` and `#sp-observed-at`, the three clauses the answer wrote.
7. `meshwork spec audit docs/PROTOCOL.md` shows `re-open candidates (1)`: `$S4`. The answer rewrote the conflict clause S4 pinned at day 0. The replay asserts the row. `$S4` is looked up by its title with `q`, since day 0 minted it.
8. Print the `sp-conflict-resolution` section. Last-writer-wins by timestamp still holds; only its caveat changed, so `meshwork cover $S4 --repin` rather than a reopen.
9. `meshwork close $ANSWER`.
10. Hand off per-field merge:
    ```
    meshwork set $PFM --handoff "notesync v0.2.0 stamps changes with hybrid logical clocks. Per-field merge should compare per-field HLC stamps, never wall-clock time — PROTOCOL.md, Change timestamps."
    ```
11. Commits:
    - `test(protocol): …`
    - `feat(protocol)!: stamp changes with hybrid logical clocks`, with a `BREAKING CHANGE:` footer naming the timestamp semantics
    - `chore(release): v0.2.0`
    - `chore(store): answer notes' causal-ordering ask; repin S4; hand off per-field merge`
12. Tag `v0.2.0` and `story/2-answer`. In record mode, push before Beat 3, because the cli repo fetches `v0.2.0` from GitHub.

**PROTOCOL.md at v0.2.0.** `sp-changes` and `sp-sync` are unchanged.
```
## Change timestamps {#sp-change-timestamp}
A change's timestamp is a hybrid logical clock reading: the later of the authoring device's wall clock and the newest timestamp that device has stored, plus a counter that breaks ties. A change therefore sorts after every change its author had stored when making it. The timestamp is not the time the change was made and can run ahead of any device's clock; to show when something happened, use observed_at.

## Conflict resolution {#sp-conflict-resolution}
When two changes replace the same document, the one with the higher timestamp wins; equal timestamps break on device id. Resolution is order-independent: replicas that have stored the same changes agree. A change always beats the changes its author had stored; changes made without seeing each other still race on timestamp, so a fast clock can win that race.

## Observed time {#sp-observed-at}
observed_at is when this replica first stored a change, by this replica's clock. It is local and never sent between replicas. Logs written by v0.1 have no observed column; their observed_at is their timestamp.
```

### Beat 3 — notes-2 (meshwork-demo-notes-cli)

Author `claude (notes-2)`.

1. `meshwork prime`. It shows:
   - `spec moved under 1 live task ($ASK)`
   - `doing $CASE … [claimed: claude (notes-1)]`
   - under `asks out (1)`: `$ASK → meshwork-demo-notes-sync … answered-by meshwork-demo-notes-sync#$ANSWER (done)`

   The replay asserts all three lines appear. Then `meshwork show $ASK` prints notes-1's handoff, and the replay asserts its first sentence.
2. Print the moved clause from `../meshwork-demo-notes-sync/docs/PROTOCOL.md`, the `sp-change-timestamp` section.
3. `meshwork portfolio spec audit meshwork-demo-notes-sync#docs/PROTOCOL.md` shows:
   - `stale (1)`: `$ASK`
   - `re-open candidates (1)`: `$LABEL` (sync-1 repinned S4 in Beat 2)

   The replay asserts both rows.
4. Patch `3a-cli-notesync-v0.2`: tag `v0.2.0` in Cargo.toml, and the two `#[ignore = "waits on @ASK@"]` lines removed. Then `cargo build` once, so the v0.2.0 fetch and the `Cargo.lock` update happen in the replay, not inside a verify run, whose environment keeps only `PATH`, `HOME`, `CARGO_HOME` and `TMPDIR`. `Cargo.lock` is committed with the bump.
5. `meshwork cover $ASK --repin`, then `meshwork close $ASK`. The contract test is green.
6. `meshwork reopen $LABEL`, then `meshwork set $LABEL --parent $CASE`.
7. Patch `3b-cli-label-test`: `edited_label_never_reads_ahead`. The phone stores the laptop's fast-stamped change, then writes its own; its `list` must not read "edited in …". The replay runs the test once to show the failure line, which reads `edited in 5 minutes`.
8. `meshwork start $LABEL` — red.
9. Patch `3c-cli-label-observed`: `list` labels with `doc.observed_at_ms()`.
10. `meshwork cover $LABEL --repin`, then `meshwork close $LABEL`.
11. `meshwork close $CASE` — the re-enactment is green.
12. `meshwork tree $CASE` — the case, the ask and the label, all done.
13. Commits:
    - `fix(deps): move to notesync v0.2.0 so an edit beats the changes it had seen`
    - `fix(list): show edit times from when this device stored the change`
    - `chore(store): close the gate-rewrite case`
14. Tag `story/3-resolved`.

### Epilogue — the record

No session. These queries use `<start>`, the stamp captured when Beat 1 began.
- `meshwork portfolio q "SELECT gid, date, to_status, note FROM log WHERE to_status IS NOT NULL AND date >= '<start>' ORDER BY date, gid"`
- `meshwork portfolio q "SELECT gid, author, text FROM comments WHERE date >= '<start>' ORDER BY date"`

The second query shows the receipts: the smoking gun, then the retraction.

## The replay

`story/replay.sh` performs the beats. Its two modes share one implementation.

- **Default (the demo).**
  - Makes a temp dir and fully clones the three repos over https. Provenance judges every commit that touched a task file, and a shallow clone truncates that history; how provenance reads a truncated history is untested, so the replay uses full clones.
  - Resets the cli and sync clones to day 0: `git checkout -B main story/0-day0`, then deletes `v0.2.0` and every `story/*` tag but `story/0-day0` locally. A full clone carries the recorded future, and without the reset Beat 2's `git tag v0.2.0` fails on the existing tag.
  - Writes `repos.local.toml` mapping the three names to the temp paths and exports `MESHWORK_PORTFOLIO`.
  - Performs Beats 1–3 and the epilogue with local commits only, then deletes the temp dir on exit.
  - Beat 3 builds the cli against the recorded `v0.2.0` that cargo fetches from GitHub, not against the replay's own Beat 2. Both come from patches 2a and 2b, so the code matches.
  - `--from DIR` uses existing sibling checkouts instead of cloning.
- **`--record`.**
  - Runs on fresh clones of the real repos at `story/0-day0`. Regenerates and commits the fixture first, then commits and tags as the beats specify, and pushes after each beat.
  - Re-recording means force-pushing `main`, the story tags and sync's `v0.2.0` back to `story/0-day0`. Do that only before the reveal.
- **Binary.** The release the clones pin: `~/.meshwork/versions/<tag>/meshwork` for the tag in their `.meshwork-version`, fetched over https when absent, as the skill's install reference does. `MESHWORK_BIN` overrides it, to check a candidate build against the story before a release. The replay never calls the repos' shims.
- **Helpers:**
  - `say` prints narration in plain voice.
  - `show` echoes a command the way it would be typed.
  - `mw` runs a verb.
  - `mw_refused` exits non-zero if an expected refusal passes.
  - `mint` runs `add` and captures the id.
  - `patch` writes this run's ids in place of the placeholders (`@CASE@`, `@ASK@` and the like), runs `git apply`, and prints the diffstat. Patches never carry a literal id: ids mint from entropy, so no replay mints the recording's ids.
  - `session` sets `MESHWORK_AUTHOR` and prints a banner naming the session and its repo.
  - `expect` greps captured output for a required line and fails the replay if it is missing.
- **Assertions:** every refusal, every red-check, and the prime and audit lines named in the beats, plus `lint` exit 0 per beat. A meshwork change that breaks the story fails the replay loudly.
- **Requirements:** bash, git, cargo via rustup, and network for the clones and the cargo git fetch of notesync. Runtime is about one to two minutes, mostly cargo builds of two std-only crates.
- `--pause` waits for Enter between beats, for live presentation.

## meshwork integration

**The full demo.** `scripts/demo-full.sh` is a new wrapper:
- `git clone` the portfolio repo over https into a temp dir, with no `gh`;
- run `story/replay.sh`, passing `MESHWORK_BIN` through when it is set.

CLAUDE.md gains a line for it in the same commit, saying it needs the network. The meshwork gate never runs it (zero-network gate).

What it shows beyond the small demo:
- the handoff, the refused close, `attach`, `why`, `tree` and `search`;
- cross-repo `needs`, and the ask with its answer;
- cross-repo spec pins with drift and a re-open candidate, `reopen`, `cover --repin` and `portfolio spec audit`.

It shows no `init`, because that happens in each repo's day-0 history. It shows no `close --approve`, because no human is present and its `run` verifies need no approval: task files ride store-only commits.

**The small demo.** `scripts/demo.sh` stays offline and runs on one scratch repo. `README.md:74` says it plays the quick-start loop, so it should match that loop. Three changes get it there:
- **Use DSL verifies, as the quick-start does.** Today it uses shell (`test -f repro.log`).
- **Drop `close --approve`.** `add` already approves a verify typed through the CLI, so the flag does nothing there.
- **Replace the second task's `true` verify** with one that fails until the work is done. `true` trips lint's `verify-trivial` and start's "already green" warning.

It should also end by printing the archived task file, as the quick-start does.

**README.** The full demo needs a sentence in the README pointing at `scripts/demo-full.sh`. Its wording is the owner's.

## Work breakdown

The order below is the dependency order. Ids are suggestions for whoever files these; verifies are the closing checks.

**meshwork store**

| Id | Work | Depends on | Verify |
|---|---|---|---|
| M1 | Add `scripts/demo-full.sh`, the clone-and-replay wrapper; CLAUDE.md line | R6 | `contains scripts/demo-full.sh /story\/replay\.sh/` |
| M2 | Propose the README sentence for the full demo to the owner | M1 | owner-gated |
| M3 | Make `scripts/demo.sh` play the quick-start loop it claims: DSL verifies, no `--approve`, no `true` verify, end on the archived file | — | `all(lacks scripts/demo.sh /--approve/, lacks scripts/demo.sh /verify "true"/, contains scripts/demo.sh /docs\/meshwork\/archive/)` |

**GitHub**

| Id | Work | Depends on |
|---|---|---|
| G1 | Create the three public repos under `jbrjake` | — |

**sync repo**: S1–S9 in order (S8 tags `v0.1.0`), then the backlog filed, then tag `story/0-day0`. Depends on G1.

**cli repo**: C1–C7 and LABEL, then the backlog filed (TITLE needs PFM's id), then tag `story/0-day0`. Depends on `v0.1.0` and R1, because LABEL's pin and TITLE's need resolve through the demo registry.

**portfolio repo** (its own store)

| Id | Work | Depends on | Verify |
|---|---|---|---|
| R1 | Register the three repos; gitignore `repos.local.toml`; store with alias `pf` | G1 | `all(contains repos.toml /meshwork-demo-notes-cli/, contains repos.toml /meshwork-demo-notes-sync/, contains .gitignore /repos\.local\.toml/)` |
| R2 | Generate the gate-rewrite fixture with `fixture-gen` on notesync v0.1.0 | sync day-0 | `all(exists story/fixtures/gate-rewrite/laptop-changes.log, exists story/fixtures/gate-rewrite/phone-changes.log)` |
| R3 | Author the beat patches against day-0 on scratch branches | R2, cli and sync day-0 | `all(exists story/patches/1a-cli-reenact.patch, exists story/patches/2b-sync-hlc.patch, exists story/patches/3c-cli-label-observed.patch)` |
| R4 | Write the session texts (case, diagnosis, ask, handoffs) | — | `exists story/text/handoff.md` |
| R5 | Write `story/replay.sh` (both modes, helpers, assertions) | R3, R4 | `contains story/replay.sh /mw_refused/` |
| R6 | Record the story; push; tag | R5 | `contains story/recording.md /story\/3-resolved/` |
| R7 | Portfolio README: what the three repos are, staged disclosure, how to replay | R6 | `contains README.md /story\/replay\.sh/` |

The cli and sync READMEs carry the same disclosure from day 0 (S9, C7).

R3 detail: each patch applies cleanly to the state its beat leaves, and names ids only through placeholders. Patches 3b and 3c need notesync v0.2.0 before it exists on GitHub. Author them against a local sync checkout with 2b applied, through a scratch `[patch]` in a scratch cargo config; it is never committed.

## Risks

- **Network.** The demo clones from GitHub and cargo fetches notesync by tag. If GitHub is down, the clone fails at the first step with git's own error.
- **Provenance needs clean history.** A day-0 commit that mixes task files with code gates that task's `run` verifies, which would stall the story at an approval prompt nobody is there to answer. Shallow clones are untested against provenance, so the replay never makes one.
- **`target/` placement.** meshwork's verify runner passes only `PATH`, `HOME`, `CARGO_HOME` and `TMPDIR`, so `CARGO_TARGET_DIR` never reaches verify builds and `target/` lands in each clone. It is a temp dir in the replay. Under `~/Documents`, each crate's gate re-creates `target/.metadata_never_index` so Spotlight skips the build output, as leras's `scripts/mark-target-unindexed.sh` does; `mdutil -i off` works per volume, not per directory.
- **Story regressions.** Changes to meshwork output can break the replay's assertions. That is the alarm working: run the replay with `MESHWORK_BIN` set to a candidate build before a release.
