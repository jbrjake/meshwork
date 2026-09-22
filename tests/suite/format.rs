//! `format::` — FORMAT.md contract checks that aren't parser or lint
//! specifics: value constraints the config table promises (mw-a6jdf5s),
//! and the conformance corpus a third-party reader self-checks against
//! (mw-7c6svyn).

use meshwork::lint::{lint_store, Severity};
use meshwork::store::load_repo;

/// mw-a6jdf5s: ID recovery from an invalid file takes the first two
/// dash-segments of the stem, so a dashed alias (`my-repo`) silently
/// corrupts recovery. The alias is `[a-z0-9]+` — lint errors on anything
/// else; `init` refuses to write one.
#[test]
fn alias_charset() {
    assert!(meshwork::id::valid_alias("mw"));
    assert!(meshwork::id::valid_alias("sazed42"));
    for bad in ["my-repo", "MW", "", "a_b", "a.b", "é"] {
        assert!(!meshwork::id::valid_alias(bad), "accepted `{bad}`");
    }

    let dir = tempfile::tempdir().unwrap();
    let mw = dir.path().join("repo/docs/meshwork");
    std::fs::create_dir_all(&mw).unwrap();
    std::fs::write(mw.join("config.toml"), "alias = \"my-repo\"\n").unwrap();
    let f = lint_store(&load_repo(&dir.path().join("repo")).unwrap());
    assert!(
        f.iter().any(|f| f.severity == Severity::Error
            && f.code == "alias-charset"
            && f.message.contains("my-repo")),
        "{f:?}"
    );
}

/// The five deterministic projection tables over the conformance store,
/// column order and sort as documented in `fixtures/conformance/README.md`.
/// `error` is pinned as presence only — its text is reader-defined.
const CONFORMANCE_QUERIES: [(&str, &str); 5] = [
    (
        "tasks",
        "SELECT gid, repo, id, title, status, category, verify, waived, seq, \
         created, blocked_reason, claimed_by, github, addressed_to, path, \
         (error IS NOT NULL) AS has_error, body, handoff, parent \
         FROM tasks ORDER BY gid",
    ),
    (
        "edges",
        "SELECT src_gid, dst_gid, kind, resolved FROM edges \
         ORDER BY src_gid, kind, dst_gid",
    ),
    (
        "labels",
        "SELECT gid, label FROM labels ORDER BY gid, label",
    ),
    (
        "comments",
        "SELECT gid, ord, date, author, text, hash FROM comments ORDER BY gid, ord",
    ),
    (
        "log",
        "SELECT gid, ord, date, from_status, to_status, note FROM log ORDER BY gid, ord",
    ),
];

/// mw-7c6svyn: the FORMAT.md conformance corpus — a golden store plus its
/// expected projection, self-contained under `fixtures/conformance/`, so a
/// third-party reader implemented from the spec can diff itself against
/// the same bytes. Where this corpus and FORMAT.md disagree, the spec
/// wins and either the corpus or the binary has a bug.
#[tokio::test]
async fn conformance_corpus() {
    use crate::common::{fixtures_root, sql_rows};
    use meshwork::tables::session_for;

    let corpus = fixtures_root().join("conformance");
    let store = load_repo(&corpus).unwrap();
    let ctx = session_for(&[store], &[]).unwrap();

    let mut doc = serde_json::Map::new();
    for (table, sql) in CONFORMANCE_QUERIES {
        let rows = sql_rows(&ctx, sql).await;
        doc.insert(table.to_string(), serde_json::json!(rows));
    }
    let actual = serde_json::to_string_pretty(&serde_json::Value::Object(doc)).unwrap() + "\n";

    let expected_path = corpus.join("expected.json");
    if std::env::var_os("MESHWORK_BLESS").is_some() {
        std::fs::write(&expected_path, &actual).unwrap();
        eprintln!("blessed conformance expected.json — review the diff against FORMAT.md");
    }
    let expected = std::fs::read_to_string(&expected_path).unwrap_or_else(|_| {
        panic!("missing conformance expected.json; generate with MESHWORK_BLESS=1")
    });
    assert_eq!(
        expected, actual,
        "conformance drift: projection no longer matches fixtures/conformance/expected.json \
         (if FORMAT.md sides with the new output: MESHWORK_BLESS=1, then review)"
    );
}

/// mw-hmg3f3d: `blocked-reason` is required WHEN blocked — the only-if
/// half is dropped. A task that was blocked, got unblocked, and kept its
/// reason is a store a human calls fine: it parses valid and draws no
/// error (the binary never enforced the iff; the spec text was the bug —
/// a third-party writer implementing it would fail healthy stores).
#[test]
fn stale_blocked_reason_legal() {
    let dir = tempfile::tempdir().unwrap();
    let mw = dir.path().join("repo/docs/meshwork");
    std::fs::create_dir_all(&mw).unwrap();
    std::fs::write(mw.join("config.toml"), "alias = \"zz\"\n").unwrap();
    std::fs::write(
        mw.join("zz-stl1-once-blocked.md"),
        "---\nid: zz-stl1\ntitle: Once blocked\nstatus: open\nverify: \"true\"\n\
         blocked-reason: waiting on the rig — resolved since\n---\nx\n",
    )
    .unwrap();
    let store = load_repo(&dir.path().join("repo")).unwrap();
    assert!(
        store.entries.iter().any(|e| matches!(
            &e.parsed,
            meshwork::parse::ParsedTask::Valid(t) if t.id == "zz-stl1"
        )),
        "a stale blocked-reason must parse valid"
    );
    let f = lint_store(&store);
    assert!(
        !f.iter()
            .any(|x| x.severity == Severity::Error && x.subject == "zz-stl1"),
        "never an error: {f:?}"
    );

    // The spec side: FORMAT.md states required-when-blocked, not iff.
    let spec =
        std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("FORMAT.md"))
            .unwrap();
    assert!(
        !spec.contains("iff `status: blocked`"),
        "FORMAT.md still demands the only-if half"
    );
    assert!(
        spec.contains("required non-empty when `status: blocked`"),
        "FORMAT.md lost the required-when-blocked rule"
    );
}

/// mw-5rgq9ka: one contract, one number — the `--json` envelope's
/// `meshwork.schema` IS the store format version, observed from a real
/// invocation, and FORMAT.md states the mapping (a reader otherwise
/// meets `format = 1` on disk and `schema` in output with no stated
/// relationship).
#[test]
fn version_matches_envelope() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("work");
    std::fs::create_dir_all(&repo).unwrap();
    assert!(std::process::Command::new("git")
        .args(["init", "-q"])
        .current_dir(&repo)
        .status()
        .unwrap()
        .success());
    let mw = |args: &[&str]| {
        let mut cmd = assert_cmd::Command::cargo_bin("meshwork").unwrap();
        // Hermetic per the suite's rule (mw-k7r5): HOME inside the
        // tempdir so no real portfolio registry can reach the test.
        cmd.current_dir(&repo)
            .env("HOME", dir.path())
            .env("MESHWORK_TRUST", "1")
            .env_remove("MESHWORK_PORTFOLIO");
        cmd.args(args);
        cmd
    };
    mw(&["init"]).assert().success();
    let out = mw(&["lint", "--json"]).assert().success();
    let text = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let envelope: serde_json::Value =
        serde_json::from_str(text.lines().next().expect("one envelope line")).unwrap();
    assert_eq!(
        envelope["meshwork"]["schema"].as_u64(),
        Some(meshwork::store::STORE_FORMAT),
        "envelope schema and store format are the same number: {envelope}"
    );

    // The spec side of the pin: FORMAT.md must name the envelope and tie
    // its `schema` to the format version.
    let spec =
        std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("FORMAT.md"))
            .unwrap();
    assert!(
        spec.contains("envelope") && spec.contains("`schema`"),
        "FORMAT.md never states the envelope `schema` ≡ format mapping"
    );
}

/// mw-1byhnj1: the close anchor gets a normative extraction pattern —
/// one agreed regex instead of one per consumer. FORMAT.md quotes it
/// verbatim, the pattern extracts what a real close mints (real git,
/// real short sha), and near-miss shapes carry no anchor.
#[test]
fn close_anchor_pattern() {
    const PATTERN: &str = " @ ([0-9a-f]{4,40})(?:\\+([1-9][0-9]*))?$";

    // Spec side: FORMAT.md states exactly this pattern.
    let spec =
        std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("FORMAT.md"))
            .unwrap();
    assert!(
        spec.contains(PATTERN),
        "FORMAT.md must quote the normative anchor pattern `{PATTERN}`"
    );

    // Binary side: a real close in a repo with history mints a →done note
    // this pattern extracts. The store edit itself is uncommitted at close
    // time, so the dirty `+N` arm is the one exercised.
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("work");
    std::fs::create_dir_all(&repo).unwrap();
    crate::common::git(&repo, &["init", "-q"]);
    crate::common::git(&repo, &["config", "user.name", "Fixture User"]);
    crate::common::git(&repo, &["config", "user.email", "fixture@example.invalid"]);
    let run = |args: &[&str]| {
        let mut cmd = assert_cmd::Command::cargo_bin("meshwork").unwrap();
        cmd.current_dir(&repo)
            .env("MESHWORK_TRUST", "1")
            .env("HOME", dir.path())
            .env_remove("MESHWORK_PORTFOLIO");
        cmd.args(args).assert().success()
    };
    run(&["init"]);
    crate::common::git(&repo, &["add", "-A"]);
    crate::common::git(&repo, &["commit", "-qm", "seed"]);
    let out = run(&["add", "Anchored", "--verify", "true"]);
    let id = String::from_utf8(out.get_output().stdout.clone())
        .unwrap()
        .lines()
        .next()
        .unwrap()
        .to_string();
    run(&["close", &id]);

    let archive = repo.join("docs/meshwork/archive");
    let file = std::fs::read_dir(&archive)
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| {
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with(&format!("{id}-"))
        })
        .unwrap();
    let text = std::fs::read_to_string(&file).unwrap();
    let done = text.lines().find(|l| l.contains("→done")).unwrap();

    let re = regex::Regex::new(PATTERN).unwrap();
    let caps = re
        .captures(done)
        .unwrap_or_else(|| panic!("minted →done note carries no extractable anchor: {done}"));
    let head = {
        let out = std::process::Command::new("git")
            .args(["rev-parse", "--short", "HEAD"])
            .current_dir(&repo)
            .output()
            .unwrap();
        String::from_utf8(out.stdout).unwrap().trim().to_string()
    };
    assert_eq!(&caps[1], head, "group 1 is the short sha as git minted it");
    let dirty: u64 = caps
        .get(2)
        .expect("dirty tree mints +N")
        .as_str()
        .parse()
        .unwrap();
    assert!(dirty >= 1, "+N counts paths, minted only when > 0");

    // Clean-tree form: no +N, still an anchor.
    let clean = re.captures("2026-08-09 doing→done — verify exit 0 @ ab12cd3");
    assert_eq!(&clean.unwrap()[1], "ab12cd3");

    // Near-misses carry no anchor — not an error, just absence.
    for miss in [
        "verify exit 0 @ 3F5FF64",     // uppercase is never minted
        "verify exit 0 @ abc",         // below git's 4-char floor
        "verify exit 0 @ 3f5ff64+0",   // +0 is never minted (clean omits)
        "verify exit 0 @ 3f5ff64 now", // not note-terminal
        "verify exit 0",               // no anchor at all
    ] {
        assert!(
            re.captures(miss).is_none(),
            "`{miss}` must not read as anchored"
        );
    }
}

/// mw-8x954nr: the spec rules the stamp middle ground. Conforming forms
/// are exactly date-only and UTC-Z minute; anything else with a date
/// prefix (an offset stamp, seconds) is nonconforming — never minted,
/// compared as opaque text as written. Prefix-ordering is stated, not
/// implied: date-only is a strict prefix of its day's minute stamps, so
/// it sorts before them and max-stamp prefers the more precise entry.
#[test]
fn stamp_ordering() {
    // Spec side: FORMAT.md states the offset ruling and the ordering.
    let spec =
        std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("FORMAT.md"))
            .unwrap();
    assert!(
        spec.contains("nonconforming") && spec.contains("opaque text"),
        "FORMAT.md must rule offset stamps nonconforming, compared as opaque text"
    );
    assert!(
        spec.contains("date-only sorts before"),
        "FORMAT.md must state the prefix-ordering property explicitly"
    );

    // The property itself, in executable form: a strict prefix sorts first.
    assert!("2026-08-06" < "2026-08-06T21:47Z");

    // Behavior side: last-activity is the as-written lexicographic max.
    // The offset stamp is civil 2026-08-07T01:47Z — LATER than the Z
    // entry — but compares as opaque text (`-` < `Z`), so the conforming
    // stamp stays the max. That is the documented cost of minting one.
    let text = "---\nid: az-stmp1\ntitle: Stamps\nstatus: doing\n---\nbody\n\n\
                ## log\n\
                - 2026-08-06 created\n\
                - 2026-08-06T21:47Z open→doing\n\
                - 2026-08-06T21:47-04:00 hand-edited with a local offset\n";
    let t = match meshwork::parse::parse_task_str("az-stmp1-stamps.md", text) {
        meshwork::parse::ParsedTask::Valid(t) => *t,
        meshwork::parse::ParsedTask::Invalid(inv) => panic!("valid file: {}", inv.error),
    };
    assert_eq!(
        t.last_activity_date().as_deref(),
        Some("2026-08-06T21:47Z"),
        "conforming max wins; the offset form participates as opaque text"
    );
}

/// mw-e60thg2: the log grammar says token one is the date, as written —
/// unconditionally — while the projection promised `date` NULL "if the
/// entry has none", unreachable under that grammar. Ruled for the
/// grammar: token one is always the date, even when it isn't
/// date-shaped; NULL only for an entry with no text at all. The
/// conformance corpus carries the deciding fixture.
#[test]
fn log_date_nullability() {
    use meshwork::parse::parse_log_line;

    // Spec side: the unreachable NULL clause is gone, the ruling stated.
    let spec =
        std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("FORMAT.md"))
            .unwrap();
    assert!(
        !spec.contains("NULL if the entry has none"),
        "FORMAT.md still carries the unreachable NULL clause"
    );
    assert!(
        spec.contains("even when it isn't date-shaped"),
        "FORMAT.md must state that token one is the date unconditionally"
    );

    // Behavior side: a dateless hand-written entry projects its first
    // token as the date — garbage in, garbage visible in SQL.
    let e = parse_log_line("fixed the thing");
    assert_eq!(e.date.as_deref(), Some("fixed"));
    assert_eq!(e.note.as_deref(), Some("the thing"));

    // NULL date exists exactly once: the entry with no text at all.
    let empty = parse_log_line("");
    assert_eq!(empty.date, None);
    assert_eq!(empty.note, None);

    // Corpus side: the deciding fixture exists and projects as ruled.
    let corpus = crate::common::fixtures_root().join("conformance");
    let fixture = std::fs::read_to_string(
        corpus.join("docs/meshwork/cf-l0gedge-log-and-comment-grammar-edges.md"),
    )
    .unwrap();
    assert!(
        fixture.contains("\n- fixed the thing\n"),
        "cf-l0gedge must carry the dateless free-text entry"
    );
    let expected = std::fs::read_to_string(corpus.join("expected.json")).unwrap();
    assert!(
        expected.contains(r#""fixed""#),
        "expected.json must project the first token as the date"
    );
}

/// MW-T2: a clause is named by its anchor id and hashed without its
/// heading line, so retitling the heading breaks nothing and moves
/// nothing; the words under it are what a pin is a pin of. A clause ref
/// is its own syntax — a heading slug is not one.
#[test]
fn clause_ref_survives_heading_edit() {
    use meshwork::spec::{clause, parse_ref, sha_hex};
    let before = "# Spec\n\n## Close gate {#sp-close-gate}\n\nA task closes on exit 0.   \n\n\n## Next\n\nother\n";
    let retitled = before.replace("## Close gate {", "## 4.1 The close gate, restated {");
    let reworded = before.replace("exit 0", "exit 0 only");
    let a = clause(before, "sp-close-gate").unwrap();
    let b = clause(&retitled, "sp-close-gate").unwrap();
    let c = clause(&reworded, "sp-close-gate").unwrap();
    assert_eq!(
        a.text, "A task closes on exit 0.",
        "right-trimmed, outer blanks dropped"
    );
    assert_eq!(a.sha, sha_hex(&a.text));
    assert_eq!(
        a.sha, b.sha,
        "a retitled heading is the same clause at the same hash"
    );
    assert_eq!(b.heading, "4.1 The close gate, restated");
    assert_ne!(a.sha, c.sha, "changed words move the hash");
    assert!(
        clause(before, "sp-next").is_none(),
        "an unanchored heading is no clause"
    );
    assert!(parse_ref("docs/SPEC.md#sp-close-gate").is_ok());
    assert!(parse_ref("docs/SPEC.md#§-close-gate").is_err());
    assert!(parse_ref("docs/SPEC.md#close-gate").is_err());
}
