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
