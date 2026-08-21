//! `lint::` — unit tier for the lint engine (PLAN 0.9; MW-A5/B2/B3/B7/K3),
//! running `meshwork::lint::lint_store` over corpus and constructed stores.

use crate::common::fixtures_root;
use meshwork::lint::{lint_store, Severity};
use meshwork::store::load_repo;

fn broken_findings() -> Vec<meshwork::lint::Finding> {
    lint_store(&load_repo(&fixtures_root().join("alpha-broken")).unwrap())
}

fn has(findings: &[meshwork::lint::Finding], severity: Severity, code: &str, needle: &str) -> bool {
    findings.iter().any(|f| {
        f.severity == severity
            && f.code == code
            && format!("{} {}", f.subject, f.message).contains(needle)
    })
}

/// MW-B2: needs-cycles are lint errors.
#[test]
fn cycle_needs() {
    let f = broken_findings();
    assert!(has(&f, Severity::Error, "cycle-needs", "ax-cyc1"), "{f:?}");
}

/// MW-B2: parent-cycles are lint errors.
#[test]
fn cycle_parent() {
    let f = broken_findings();
    assert!(has(&f, Severity::Error, "cycle-parent", "ax-pcy1"), "{f:?}");
}

/// MW-B3: `parent` never crosses repos.
#[test]
fn parent_crossrepo_error() {
    let f = broken_findings();
    assert!(
        has(&f, Severity::Error, "parent-crossrepo", "ax-xrp1"),
        "{f:?}"
    );
}

/// MW-A5: descriptions over the ~2KB byte budget warn (never a line count).
#[test]
fn description_size_warn() {
    let dir = tempfile::tempdir().unwrap();
    let mw = dir.path().join("repo/docs/meshwork");
    std::fs::create_dir_all(&mw).unwrap();
    std::fs::write(mw.join("config.toml"), "alias = \"zz\"\n").unwrap();
    let big = "long design narrative that belongs behind docs: links. ".repeat(60);
    assert!(big.len() > 2048);
    std::fs::write(
        mw.join("zz-big1-oversized.md"),
        format!("---\nid: zz-big1\ntitle: Oversized\nstatus: open\nverify: \"true\"\n---\n{big}\n"),
    )
    .unwrap();
    let f = lint_store(&load_repo(&dir.path().join("repo")).unwrap());
    assert!(
        has(&f, Severity::Warning, "description-size", "zz-big1"),
        "{f:?}"
    );
}

/// MW-K3: attachments >1MB warn (excerpt-first culture).
#[test]
fn attachment_size_warn() {
    let f = lint_store(&load_repo(&fixtures_root().join("alpha")).unwrap());
    assert!(
        has(&f, Severity::Warning, "attachment-size", "az-a7t2"),
        "{f:?}"
    );
}

/// MW-B7: a done parent with live children warns — rollup is advisory,
/// never auto-close.
#[test]
fn parent_rollup_warn() {
    let dir = tempfile::tempdir().unwrap();
    let mw = dir.path().join("repo/docs/meshwork");
    std::fs::create_dir_all(&mw).unwrap();
    std::fs::write(mw.join("config.toml"), "alias = \"zz\"\n").unwrap();
    std::fs::write(
        mw.join("zz-par1-parent.md"),
        "---\nid: zz-par1\ntitle: Parent\nstatus: done\nverify: \"true\"\n---\nx\n",
    )
    .unwrap();
    std::fs::write(
        mw.join("zz-chd1-child.md"),
        "---\nid: zz-chd1\ntitle: Child\nstatus: open\nparent: zz-par1\nverify: \"true\"\n---\nx\n",
    )
    .unwrap();
    let f = lint_store(&load_repo(&dir.path().join("repo")).unwrap());
    assert!(
        has(&f, Severity::Warning, "parent-rollup", "zz-par1"),
        "{f:?}"
    );
}

/// mw-nfv26ss: a session flipped `status:` with a raw edit — no verify
/// ran, no transition logged. Hand edits are legal; the catchable
/// signature is a terminal status whose log has no matching `→status`
/// entry. A real close/drop always writes one, and import provenance is
/// a legitimate birth certificate for tasks born terminal.
#[test]
fn status_flip_without_log() {
    let dir = tempfile::tempdir().unwrap();
    let mw = dir.path().join("repo/docs/meshwork");
    std::fs::create_dir_all(&mw).unwrap();
    std::fs::write(mw.join("config.toml"), "alias = \"zz\"\n").unwrap();
    std::fs::write(
        mw.join("zz-flp1-handflipped.md"),
        "---\nid: zz-flp1\ntitle: Hand-flipped\nstatus: done\nverify: \"true\"\n---\n\n## log\n- 2026-08-12 created\n",
    )
    .unwrap();
    std::fs::write(
        mw.join("zz-flp2-handdropped.md"),
        "---\nid: zz-flp2\ntitle: Hand-dropped\nstatus: dropped\nverify: \"true\"\n---\n\n## log\n- 2026-08-12 created\n",
    )
    .unwrap();
    std::fs::write(
        mw.join("zz-cls1-closed.md"),
        "---\nid: zz-cls1\ntitle: Really closed\nstatus: done\nverify: \"true\"\n---\n\n## log\n- 2026-08-12 created\n- 2026-08-13 open→done — verify exit 0\n",
    )
    .unwrap();
    std::fs::write(
        mw.join("zz-imp1-imported.md"),
        "---\nid: zz-imp1\ntitle: Imported done\nstatus: done\nverify: \"true\"\n---\n\n## log\n- 2026-08-12 imported from TODO.md\n",
    )
    .unwrap();
    let f = lint_store(&load_repo(&dir.path().join("repo")).unwrap());
    assert!(
        has(&f, Severity::Warning, "status-unlogged", "zz-flp1"),
        "{f:?}"
    );
    assert!(
        has(&f, Severity::Warning, "status-unlogged", "zz-flp2"),
        "{f:?}"
    );
    assert!(
        !has(&f, Severity::Warning, "status-unlogged", "zz-cls1"),
        "{f:?}"
    );
    assert!(
        !has(&f, Severity::Warning, "status-unlogged", "zz-imp1"),
        "{f:?}"
    );
}

/// The kitchen-sink corpus is error-free: its only findings are the two
/// planted warnings (no-verify spike, >1MB attachment).
#[test]
fn alpha_corpus_error_free() {
    let f = lint_store(&load_repo(&fixtures_root().join("alpha")).unwrap());
    let errors: Vec<_> = f.iter().filter(|x| x.severity == Severity::Error).collect();
    assert!(errors.is_empty(), "alpha must lint clean: {errors:?}");
    assert!(has(&f, Severity::Warning, "no-verify", "az-n0v1"), "{f:?}");
}

/// DESIGN §7b: `handoff:` is the outgoing session's voice on an up-next
/// task; on a done task it is stale — lint warns (handoff-stale).
#[test]
fn handoff_on_done_warn() {
    let dir = tempfile::tempdir().unwrap();
    let mw = dir.path().join("repo/docs/meshwork");
    std::fs::create_dir_all(&mw).unwrap();
    std::fs::write(mw.join("config.toml"), "alias = \"zz\"\n").unwrap();
    std::fs::write(
        mw.join("zz-old1-finished.md"),
        "---\nid: zz-old1\ntitle: Finished\nstatus: done\nverify: \"true\"\nhandoff: |\n  stale voice from a past session\n---\nx\n",
    )
    .unwrap();
    std::fs::write(
        mw.join("zz-nxt1-upnext.md"),
        "---\nid: zz-nxt1\ntitle: Up next\nstatus: open\nverify: \"true\"\nhandoff: |\n  live voice — legal on an open task\n---\nx\n",
    )
    .unwrap();
    let f = lint_store(&load_repo(&dir.path().join("repo")).unwrap());
    assert!(
        has(&f, Severity::Warning, "handoff-stale", "zz-old1"),
        "{f:?}"
    );
    assert!(
        !has(&f, Severity::Warning, "handoff-stale", "zz-nxt1"),
        "{f:?}"
    );
    assert!(
        !has(&f, Severity::Warning, "unknown-key", "zz-nxt1"),
        "handoff is schema-known: {f:?}"
    );
}

/// mw-mtn4hp8: the committed union attribute IS the concurrency mechanism —
/// a clone that lost `.gitattributes` voids FORMAT.md's Merge semantics
/// invisibly until the first bad merge. Missing or incomplete = lint ERROR.
#[test]
fn gitattributes_union_missing() {
    let dir = tempfile::tempdir().unwrap();
    let mw = dir.path().join("repo/docs/meshwork");
    std::fs::create_dir_all(&mw).unwrap();
    std::fs::write(mw.join("config.toml"), "alias = \"zz\"\n").unwrap();

    // Absent file: error.
    let f = lint_store(&load_repo(&dir.path().join("repo")).unwrap());
    assert!(
        has(&f, Severity::Error, "gitattributes-union", ".gitattributes"),
        "{f:?}"
    );

    // Present but missing the archive pattern (pre-archive stores): error
    // naming the missing line.
    std::fs::write(mw.join(".gitattributes"), "/*.md merge=union\n").unwrap();
    let f = lint_store(&load_repo(&dir.path().join("repo")).unwrap());
    assert!(
        has(&f, Severity::Error, "gitattributes-union", "/archive/*.md"),
        "{f:?}"
    );

    // Canonical content (what init writes): clean. Extra attributes on the
    // same pattern keep the union property and stay clean too.
    std::fs::write(mw.join(".gitattributes"), meshwork::store::GITATTRIBUTES).unwrap();
    let f = lint_store(&load_repo(&dir.path().join("repo")).unwrap());
    assert!(!f.iter().any(|x| x.code == "gitattributes-union"), "{f:?}");

    std::fs::write(
        mw.join(".gitattributes"),
        "/*.md merge=union diff=md\n/archive/*.md  merge=union\n",
    )
    .unwrap();
    let f = lint_store(&load_repo(&dir.path().join("repo")).unwrap());
    assert!(!f.iter().any(|x| x.code == "gitattributes-union"), "{f:?}");
}

/// MW-F3 (PLAN 4.2, mw-8r1a): a live task's `docs:` links must resolve —
/// dead anchor and dead path each warn, distinctly. Terminal tasks are
/// history; their pointers may rot without noise.
#[test]
fn anchor_missing_warn() {
    let dir = tempfile::tempdir().unwrap();
    let mw = dir.path().join("repo/docs/meshwork");
    std::fs::create_dir_all(&mw).unwrap();
    std::fs::write(mw.join("config.toml"), "alias = \"zz\"\n").unwrap();
    std::fs::write(
        dir.path().join("repo/DESIGN-z.md"),
        "# Z\n\n## 3. Real section\n\nbody.\n",
    )
    .unwrap();
    let task = |id: &str, status: &str, link: &str| {
        format!(
            "---\nid: {id}\ntitle: T {id}\nstatus: {status}\nverify: \"true\"\n\
             docs:\n  - {link}\n---\n"
        )
    };
    for (name, body) in [
        (
            "zz-doc1-a.md",
            task("zz-doc1", "open", "DESIGN-z.md#§-3-real-section"),
        ),
        (
            "zz-doc2-b.md",
            task("zz-doc2", "open", "DESIGN-z.md#§-9-gone"),
        ),
        ("zz-doc3-c.md", task("zz-doc3", "open", "GONE.md#§-1-x")),
        (
            "archive/zz-doc4-d.md",
            task("zz-doc4", "done", "GONE.md#§-1-x"),
        ),
    ] {
        let path = mw.join(name);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, body).unwrap();
    }

    let f = lint_store(&load_repo(&dir.path().join("repo")).unwrap());
    assert!(
        has(&f, Severity::Warning, "anchor-missing", "zz-doc2"),
        "{f:?}"
    );
    assert!(
        has(&f, Severity::Warning, "doc-missing", "zz-doc3"),
        "{f:?}"
    );
    let dead = |id: &str| {
        f.iter().any(|x| {
            x.subject.contains(id) && (x.code == "anchor-missing" || x.code == "doc-missing")
        })
    };
    assert!(!dead("zz-doc1"), "resolving link must stay clean: {f:?}");
    assert!(
        !dead("zz-doc4"),
        "terminal rot is history, not noise: {f:?}"
    );
}

/// mw-221f3jt: statically trivially-satisfiable verifies warn — the golf
/// (bare `true`/`echo`/`touch`) and presence checks already green at
/// lint time. Warn-only; the start red-check (mw-175bn4c) is the
/// dynamic tier of the same defense.
#[test]
fn trivial_verify_warn() {
    let dir = tempfile::tempdir().unwrap();
    let mw = dir.path().join("repo/docs/meshwork");
    std::fs::create_dir_all(&mw).unwrap();
    std::fs::write(mw.join("config.toml"), "alias = \"zz\"\n").unwrap();
    std::fs::write(dir.path().join("repo/README.md"), "present\n").unwrap();
    let task = |id: &str, status: &str, verify: &str| {
        format!("---\nid: {id}\ntitle: T {id}\nstatus: {status}\nverify: {verify:?}\n---\n")
    };
    for (name, body) in [
        ("zz-trv1-a.md", task("zz-trv1", "open", "true")),
        ("zz-trv2-b.md", task("zz-trv2", "open", "echo done")),
        ("zz-trv3-c.md", task("zz-trv3", "open", "touch marker.txt")),
        ("zz-trv4-d.md", task("zz-trv4", "open", "test -f README.md")),
        ("zz-trv5-e.md", task("zz-trv5", "open", "exists README.md")),
        // Red today — a presence check on a file the work will create.
        (
            "zz-red1-f.md",
            task("zz-red1", "open", "test -f docs/notyet.md"),
        ),
        (
            "zz-red2-g.md",
            task("zz-red2", "open", "exists docs/notyet.md"),
        ),
        // Real command, and compound shells, stay unjudged.
        ("zz-ok1-h.md", task("zz-ok1", "open", "cargo test smoke")),
        (
            "zz-ok2-i.md",
            task("zz-ok2", "open", "test -f README.md && grep -q x README.md"),
        ),
        // Terminal tasks are history — their golf no longer matters.
        ("archive/zz-old1-j.md", task("zz-old1", "done", "true")),
    ] {
        let path = mw.join(name);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, body).unwrap();
    }

    let f = lint_store(&load_repo(&dir.path().join("repo")).unwrap());
    for warned in ["zz-trv1", "zz-trv2", "zz-trv3", "zz-trv4", "zz-trv5"] {
        assert!(
            has(&f, Severity::Warning, "verify-trivial", warned),
            "{warned} must warn: {f:?}"
        );
    }
    for clean in ["zz-red1", "zz-red2", "zz-ok1", "zz-ok2", "zz-old1"] {
        assert!(
            !has(&f, Severity::Warning, "verify-trivial", clean),
            "{clean} must stay clean: {f:?}"
        );
    }
}

/// mw-06j1wqe: doing-rot pressure. A doing task whose newest dated
/// activity is older than the staleness window warns (`doing-stale`),
/// and a doing task nobody claims warns (`doing-unclaimed`) — in the
/// field the doing list only ever grew, and nothing pushed back.
#[test]
fn stale_doing_warn() {
    let dir = tempfile::tempdir().unwrap();
    let mw = dir.path().join("repo/docs/meshwork");
    std::fs::create_dir_all(&mw).unwrap();
    std::fs::write(mw.join("config.toml"), "alias = \"zz\"\n").unwrap();
    let today = meshwork::clock::stamp();
    let task = |id: &str, status: &str, claimed: Option<&str>, log_date: &str| {
        let claim = claimed.map_or(String::new(), |c| format!("claimed-by: {c}\n"));
        format!(
            "---\nid: {id}\ntitle: T {id}\nstatus: {status}\n{claim}verify: \"true\"\n---\n\n\
             ## log\n- {log_date} created\n"
        )
    };
    for (name, body) in [
        (
            "zz-rot1-old.md",
            task("zz-rot1", "doing", Some("worker"), "2026-01-01"),
        ),
        (
            "zz-frs1-fresh.md",
            task("zz-frs1", "doing", Some("worker"), &today),
        ),
        (
            "zz-unc1-unclaimed.md",
            task("zz-unc1", "doing", None, &today),
        ),
        (
            "zz-opn1-open.md",
            task("zz-opn1", "open", None, "2026-01-01"),
        ),
    ] {
        std::fs::write(mw.join(name), body).unwrap();
    }
    let f = lint_store(&load_repo(&dir.path().join("repo")).unwrap());
    assert!(
        has(&f, Severity::Warning, "doing-stale", "zz-rot1"),
        "{f:?}"
    );
    assert!(
        !has(&f, Severity::Warning, "doing-stale", "zz-frs1"),
        "fresh activity is not rot: {f:?}"
    );
    assert!(
        has(&f, Severity::Warning, "doing-unclaimed", "zz-unc1"),
        "{f:?}"
    );
    assert!(
        !has(&f, Severity::Warning, "doing-unclaimed", "zz-rot1"),
        "claimed doing is owned: {f:?}"
    );
    for code in ["doing-stale", "doing-unclaimed"] {
        assert!(
            !has(&f, Severity::Warning, code, "zz-opn1"),
            "open tasks are the queue, not rot: {f:?}"
        );
    }
}

/// mw-t01ek6s: `cat >>` appends below the tail sections, where the parser
/// ignores content silently — lint names the stranded prose on live
/// tasks. Terminal rot is history, not noise.
#[test]
fn stray_prose_below_log() {
    let dir = tempfile::tempdir().unwrap();
    let mw = dir.path().join("repo/docs/meshwork");
    std::fs::create_dir_all(mw.join("archive")).unwrap();
    std::fs::write(mw.join("config.toml"), "alias = \"zz\"\n").unwrap();
    std::fs::write(
        mw.join("zz-cat1-damaged.md"),
        "---\nid: zz-cat1\ntitle: Damaged\nstatus: open\nverify: \"true\"\n---\n\
         Real body.\n\n## log\n- 2026-08-01 created\n\
         Appended paragraph the parser drops.\n",
    )
    .unwrap();
    std::fs::write(
        mw.join("zz-ok1-clean.md"),
        "---\nid: zz-ok1\ntitle: Clean\nstatus: open\nverify: \"true\"\n---\n\
         Body.\n\n## log\n- 2026-08-01 created\n  a legal continuation\n\n",
    )
    .unwrap();
    std::fs::write(
        mw.join("archive/zz-old1-done.md"),
        "---\nid: zz-old1\ntitle: Old\nstatus: done\nverify: \"true\"\n---\n\
         ## log\n- 2026-08-01 done\nstray but historical\n",
    )
    .unwrap();
    let f = lint_store(&load_repo(&dir.path().join("repo")).unwrap());
    assert!(
        has(&f, Severity::Warning, "stray-tail-content", "zz-cat1"),
        "{f:?}"
    );
    assert!(
        !has(&f, Severity::Warning, "stray-tail-content", "zz-ok1"),
        "legal tail lines stay clean: {f:?}"
    );
    assert!(
        !has(&f, Severity::Warning, "stray-tail-content", "zz-old1"),
        "terminal rot is history: {f:?}"
    );
}

/// mw-n3xgfs0: the mechanical repair — relocate stray tail content above
/// `## log` preserving order, losing nothing (the multiset invariant the
/// field repair had to hand-roll), leaving real entries as entries.
#[test]
fn fix_stray_body_relocation() {
    let damaged = "---\nid: zz-fix1\ntitle: Fix\nstatus: open\nverify: \"true\"\n---\n\
         Body head.\n\n## log\n- 2026-08-01 created\n\
         Between-entries paragraph.\n- 2026-08-02 open\u{2192}doing\n\
         ## notes\n- a bullet the parser ignores\nmore ignored prose\n\
         ## comments\n- 2026-08-03 [me] real comment\n";
    let (repaired, moved) = meshwork::lint_tail::relocate_stray(damaged).expect("stray found");
    assert_eq!(
        moved, 4,
        "prose + heading + its 2 ignored lines: {repaired}"
    );

    // Nothing lost, nothing invented: same non-blank lines, reordered.
    let multiset = |s: &str| {
        let mut v: Vec<String> = s
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(ToString::to_string)
            .collect();
        v.sort_unstable();
        v
    };
    assert_eq!(multiset(damaged), multiset(&repaired));

    // The relocated prose parses as body; the real entries survive.
    let dir = tempfile::tempdir().unwrap();
    let mw = dir.path().join("repo/docs/meshwork");
    std::fs::create_dir_all(&mw).unwrap();
    std::fs::write(mw.join("config.toml"), "alias = \"zz\"\n").unwrap();
    let path = mw.join("zz-fix1-fix.md");
    std::fs::write(&path, &repaired).unwrap();
    let meshwork::parse::ParsedTask::Valid(t) = meshwork::parse::parse_task_file(&path) else {
        panic!("repaired file must parse: {repaired}");
    };
    assert!(
        t.description.contains("Between-entries paragraph.")
            && t.description.contains("## notes")
            && t.description.contains("more ignored prose"),
        "stray content lives in the body now: {}",
        t.description
    );
    assert_eq!(t.log.len(), 2, "{:?}", t.log);
    assert_eq!(t.comments.len(), 1, "{:?}", t.comments);
    assert!(t.warnings.is_empty(), "clean parse: {:?}", t.warnings);

    // Idempotent: a repaired file has nothing left to move.
    assert!(meshwork::lint_tail::relocate_stray(&repaired).is_none());
}

/// mw-yyf1bab: a verify edited after this clone approved it is the
/// silent-weakening attack — lint shows approved-vs-current instead of
/// leaving the change to a close-time prompt the operator clicks through.
#[test]
fn verify_changed_since_approval() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("repo");
    let mw = root.join("docs/meshwork");
    std::fs::create_dir_all(&mw).unwrap();
    std::fs::write(mw.join("config.toml"), "alias = \"zz\"\n").unwrap();
    let task = |verify: &str| {
        format!("---\nid: zz-chg1\ntitle: Weakened\nstatus: open\nverify: {verify:?}\n---\n")
    };
    let file = mw.join("zz-chg1-weakened.md");
    std::fs::write(&file, task("run cargo test real::gate")).unwrap();
    meshwork::trust::record_approval(&root, "zz-chg1", "run cargo test real::gate").unwrap();

    let clean = lint_store(&load_repo(&root).unwrap());
    assert!(
        !clean
            .iter()
            .any(|x| x.code == "verify-changed-since-approval"),
        "matching approval stays silent: {clean:?}"
    );

    std::fs::write(&file, task("true")).unwrap();
    let f = lint_store(&load_repo(&root).unwrap());
    assert!(
        has(
            &f,
            Severity::Warning,
            "verify-changed-since-approval",
            "zz-chg1"
        ),
        "{f:?}"
    );
    // The diff is the finding: both what was approved and what stands now.
    assert!(
        has(
            &f,
            Severity::Warning,
            "verify-changed-since-approval",
            "real::gate"
        ),
        "approved text on screen: {f:?}"
    );
    assert!(
        has(
            &f,
            Severity::Warning,
            "verify-changed-since-approval",
            "true"
        ),
        "current text on screen: {f:?}"
    );

    // Terminal tasks are history — an old approval mismatch is not noise.
    std::fs::write(
        &file,
        "---\nid: zz-chg1\ntitle: Weakened\nstatus: done\nverify: \"true\"\n---\n",
    )
    .unwrap();
    let done = lint_store(&load_repo(&root).unwrap());
    assert!(
        !done
            .iter()
            .any(|x| x.code == "verify-changed-since-approval"),
        "terminal tasks stay silent: {done:?}"
    );
}
