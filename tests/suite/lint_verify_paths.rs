// `lint::` part-file (include!d from lint.rs): what `verify-path-missing`
// reads as a deliverable. Kept apart so lint.rs stays under the line cap.

/// An `exists <p>` arm in the same `all(…)` declares `p` a deliverable — a
/// file the task exists to write — so a `contains`/`lacks` arm on that
/// same `p` is the content check on the work, not a read of a file that
/// moved. Fourteen of fourteen hits on an adopting store were that shape.
/// A glob arm (`exists dir/name-*.md`) declares every name it would match.
/// A `contains` on a path no `exists` arm names still fires, and so does a
/// bare `contains` on a missing file.
#[test]
fn verify_path_missing_honors_exists_arm() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("repo");
    let mw = root.join("docs/meshwork");
    std::fs::create_dir_all(&mw).unwrap();
    std::fs::write(mw.join("config.toml"), "alias = \"zz\"\n").unwrap();
    let task = |id: &str, verify: &str| {
        std::fs::write(
            mw.join(format!("{id}-t.md")),
            format!("---\nid: {id}\ntitle: {id}\nstatus: open\nverify: \"{verify}\"\n---\n"),
        )
        .unwrap();
    };
    task(
        "zz-dlv1",
        "all(exists docs/OUT.md, contains docs/OUT.md /Q0 .*ok/)",
    );
    task(
        "zz-dlv2",
        "all(exists docs/OUT.md, lacks docs/OUT.md TODO, contains docs/OUT.md shipped)",
    );
    task(
        "zz-dlv3",
        "all(exists docs/bench-*.md, contains docs/bench-sf1.md /ms/)",
    );
    task(
        "zz-dlv4",
        "all(exists docs/OUT.md, contains docs/OTHER.md shipped)",
    );
    task("zz-dlv5", "contains docs/OUT.md shipped");
    task(
        "zz-dlv6",
        "all(exists docs/bench-*.md, contains docs/notes-sf1.md /ms/)",
    );
    let f = lint_store(&load_repo(&root).unwrap());
    for quiet in ["zz-dlv1", "zz-dlv2", "zz-dlv3"] {
        assert!(
            !has(&f, Severity::Warning, "verify-path-missing", quiet),
            "{quiet} names its deliverable with an exists arm and must stay quiet: {f:?}"
        );
    }
    for loud in ["zz-dlv4", "zz-dlv5", "zz-dlv6"] {
        assert!(
            has(&f, Severity::Warning, "verify-path-missing", loud),
            "{loud} reads a path no exists arm declares: {f:?}"
        );
    }
    assert!(
        has(&f, Severity::Warning, "verify-path-missing", "docs/OTHER.md"),
        "the undeclared path is the one named: {f:?}"
    );
}
