# Proposal: README trust-gate re-voice (mw-4r7v8vj)

Owner ruling 2026-08-24: re-voiced narrative — the close demo goes straight
to the honest objection; the gate is narrated, not demonstrated; no
merge-arrival stand-in. README words are owner-voiced, so nothing below is
landed — this is the full proposed diff plus the real-run evidence for
every transcript line. Every proposed block was captured 2026-08-24 from
target/debug/meshwork at commit 8d777c9 (replay ids mapped to the story's
ids; the map and raw captures are in the appendix).

Four README spots are touched, plus one staging note that affects
mw-5fekg2q's replay guard.

## 1. Quick-start close block (README lines 40–47)

Current (impossible: the verify was CLI-authored in this transcript, so
close runs it without ceremony — no `--approve`, no preamble):

    $ meshwork close ac-acnxdkg --approve
    approving verify for ac-acnxdkg (this clone only):
      verify: cargo test stuff::thing

    running 1 test
    test stuff::thing ... ok

    ac-acnxdkg doing→done (verify exit 0)

Proposed (same curation as today — cargo's compile/Finished/Running lines
elided):

    $ meshwork close ac-acnxdkg

    running 1 test
    test stuff::thing ... ok

    ac-acnxdkg doing→done (verify exit 0)

Staging note (matters for the line-79 "real run" promise and for
mw-5fekg2q): `start` now red-checks the verify and warns "already green"
when it passes at start. The quick-start's `start` block shows no warning,
so the scratch staging must begin with `stuff::thing` failing and the fix
must happen inside the existing `...` elision (line 39) before the close.
That staging is also the honest demo — the task's work is what turns the
test green. The replayed `start`/`comment`/`q`/archive-cat blocks came back
byte-identical to the README's current text under that staging (modulo
minted ids/timestamps, and the `store @`/`@ c601195+5` git decorations,
which need a commit in the scratch repo, as today).

## 2. "closing tasks" section (README lines 251–275)

The two-block refusal-then-approve sequence collapses into one block; the
gate moves into prose. Proposed replacement for the whole section — the
existing security paragraphs are kept nearly verbatim where still true:

---

### closing tasks

Try to close something before the work exists, and meshwork gets straight
to the objection:

    $ meshwork close sa-jt7zg9w
    meshwork: sa-jt7zg9w stays open: verify exit 1 (`test -f docs/postmortem.md`)

A task's `verify:` command must be witnessed exiting with code 0, and this
one wasn't, since the postmortem hasn't actually been written.

Note what it didn't do: ask permission to run the command. This transcript
authored that verify with `add --verify` two blocks up, and text you author
through the CLI is trusted at mint — the approval is recorded then and
there. Text that arrives any other way is a different story. Task files can
come from untrusted sources, like third-party PRs. `verify:` fields are
executed in the shell. This is not a fantastic combination for security. So
foreign shell verifies are trust-on-first-use: `close` refuses them
(`refusing unapproved verify`) until you approve the exact text with
`close --approve`, per task, and the approval is recorded per clone,
outside git, where a merge can't plant one. (Reviewed checkouts — CI,
gates — may grant `MESHWORK_TRUST=1` instead.)

What this means is the human in the loop is responsible for security. If
you accept tasks from other people, make sure you read the contents of
anything they will execute before you approve. If you just hit your enter
key to every Claude prompt, all bets are off.

---

(The refusal text quoted in prose is real — appendix probe D captures it
firing on a hand-edited verify in the same store.)

## 3. Work-loop close block (README lines 287–290)

Current close is `--approve` with a two-line preamble; both are gone the
same way. Proposed block (intro sentence and everything before the close
unchanged):

    $ meshwork start sa-nmvpyqr --as claude
    sa-nmvpyqr open→doing
    $ meshwork comment sa-nmvpyqr --as claude "cliff reproduces at batch=64k; tracks the governor wakeup interval, not batch size"
    sa-nmvpyqr: comment added as [claude]
    $ touch repro.log        # stand-in for the actual work
    $ meshwork close sa-nmvpyqr
    sa-nmvpyqr doing→done (verify exit 0)

## 4. Concepts bullet (README line 392)

Current:

> **`verify:` is untrusted input.** Nothing shells out until this clone's
> operator approves the exact text (`close --approve`; `MESHWORK_TRUST=1`
> for checkouts reviewed before the runner touched them).

Proposed:

> **`verify:` is untrusted input.** Text this clone authors through the
> CLI is approved at mint; anything arriving by merge or hand-edit doesn't
> shell out until this clone's operator approves the exact text
> (`close --approve`; `MESHWORK_TRUST=1` for checkouts reviewed before the
> runner touched them).

## 5. demo.sh (optional, cosmetic)

`scripts/demo.sh` still runs `close "$ID" --approve`. Probe B shows
`--approve` on an already-mint-approved verify is a silent no-op — output
is identical with or without the flag, so the script is not wrong, just
carrying a vestigial flag. Dropping it aligns the script with the
re-voiced story; happy to land that separately on a word.

## Appendix: replay evidence (2026-08-24, binary @ 8d777c9)

Id map: ac-c7ex9xm→ac-acnxdkg · sa-pvrf2gz→sa-nmvpyqr ·
sa-snk1v2h→sa-jt7zg9w · sa-txa0p3j→sa-38wd6se. Stores: scratch git repos
`acme` (cargo lib, `stuff::thing` staged red then fixed) and `demo`
(config.toml alias hand-set to `sa` before first add). No network, no
touch of this repo.

Probe A — close, mint-approved verify, work not done (exit 1):

    $ meshwork close sa-snk1v2h
    meshwork: sa-snk1v2h stays open: verify exit 1 (`test -f docs/postmortem.md`)

Probe B — close --approve when already mint-approved (exit 1, no
"approving" preamble — the flag is a no-op):

    $ meshwork close sa-snk1v2h --approve
    meshwork: sa-snk1v2h stays open: verify exit 1 (`test -f docs/postmortem.md`)

Probe C — work loop end-to-end (exit 0):

    $ meshwork start sa-pvrf2gz --as claude
    sa-pvrf2gz open→doing
    $ meshwork comment sa-pvrf2gz --as claude "cliff reproduces at batch=64k; tracks the governor wakeup interval, not batch size"
    sa-pvrf2gz: comment added as [claude]
    $ touch repro.log
    $ meshwork close sa-pvrf2gz
    sa-pvrf2gz doing→done (verify exit 0)

Probe D — close after the verify text was hand-edited (merge-arrival
shape; the gate still fires, exit 1):

    $ meshwork close sa-txa0p3j
    meshwork: refusing unapproved verify for sa-txa0p3j
      verify: test -f bench/spill.csv
      task files arrive via merge and are untrusted; review the command, then:
      meshwork close sa-txa0p3j --approve   (records approval for this clone)
      reviewed checkouts (CI, gates) may grant MESHWORK_TRUST=1 instead

Probe E — quick-start close, mint-approved cargo verify (exit 0; raw,
uncurated — cargo interleaves stderr status after the stdout test lines):

    $ meshwork close ac-c7ex9xm

    running 1 test
    test stuff::thing ... ok

    test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

       Compiling acme v0.1.0 (…/replay/acme)
        Finished `test` profile [unoptimized + debuginfo] target(s) in 0.09s
         Running unittests src/lib.rs (target/debug/deps/acme-…)
    ac-c7ex9xm doing→done (verify exit 0)

With `stuff::thing` staged red, `start ac-c7ex9xm --as claude` printed
exactly `ac-c7ex9xm open→doing` — no red-check warning — and the `q`
rollup and archive-cat outputs matched the README's current blocks modulo
id/timestamp/git-hash normalization.
